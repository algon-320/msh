use super::structures::*;

use nix::fcntl;
use nix::sys;
use nix::unistd;

use std::ffi;

use std::collections::HashMap;
use std::os::unix::io;
use std::path::PathBuf;

pub type ExitCode = i32;

fn error_to_string<E: std::fmt::Display>(e: E) -> String {
    format!("{}", e)
}

fn error_then_exit<E: std::fmt::Display>(e: E) -> ! {
    eprintln!("{}", error_to_string(e));
    std::process::exit(1);
}

// ファイルを開いてファイルディスクリプタを返す
fn open_file(path: &str, flag: fcntl::OFlag) -> Result<io::RawFd, String> {
    let permission = sys::stat::Mode::S_IRUSR
        | sys::stat::Mode::S_IWUSR
        | sys::stat::Mode::S_IRGRP
        | sys::stat::Mode::S_IROTH;
    fcntl::open(path, flag, permission).map_err(|e| match e {
        nix::Error::Sys(nix::errno::Errno::EEXIST) => error_to_string(format!(
            "`{}` already exists, use `>!` to overwrite it.",
            path
        )),
        e => error_to_string(e),
    })
}

impl<'a> Shell<'a> {
    pub fn new() -> Shell<'a> {
        let mut shell = Shell {
            id: 0,
            parent: None,
            command_table: HashMap::new(),
        };
        super::builtin_commands::reload_path(&mut shell, Vec::new());
        shell
    }
    pub fn fromParent(parent: &'a Shell<'a>) -> Shell<'a> {
        let mut shell = Shell {
            id: parent.id + 1,
            parent: Some(&parent),
            command_table: parent.command_table.clone(),
        };
        super::builtin_commands::reload_path(&mut shell, Vec::new());
        shell
    }
    pub fn wait(&self) -> ExitCode {
        let mut exit_status = 0;
        loop {
            if let Ok(status) = sys::wait::waitpid(unistd::Pid::from_raw(-1), None) {
                match status {
                    sys::wait::WaitStatus::Exited(_, code) => {
                        exit_status = code;
                    }
                    _ => {}
                }
                continue;
            }
            break;
        }
        exit_status
    }
    pub fn exec(&mut self, cmds: List) -> Result<ExitCode, String> {
        let mut exit_status = 0;
        for connector in cmds.0 {
            match connector {
                Connector::Continue(pipeline) => {
                    pipeline.exec(self, 0, 1)?;
                    exit_status = self.wait();
                    // 終了コードに関わらず続行
                }
                Connector::ListTerm(pipeline) => {
                    pipeline.exec(self, 0, 1)?;
                    exit_status = self.wait();
                }
            }
        }
        Ok(exit_status)
    }
}

impl PipeLine {
    pub fn exec(
        self,
        shell: &mut Shell,
        stdin: io::RawFd,
        stdout: io::RawFd,
    ) -> Result<(), String> {
        let pipeline = self.0;
        if pipeline.is_empty() {
            panic!("PipeLine must not be empty.");
        }

        let mut next_in = stdin;
        for pipe in pipeline {
            let p = unistd::pipe().map_err(error_to_string)?;

            match pipe {
                Pipe::Stdout(cmd) => {
                    cmd.exec(shell, next_in, p.1, vec![p.0])?;
                }
                Pipe::Both(cmd) => {
                    unimplemented!();
                    next_in = p.0;
                    cmd.exec(shell, next_in, p.1, vec![p.0])?;
                }
                Pipe::PipeLineTerm(cmd) => {
                    cmd.exec(shell, next_in, stdout, Vec::new())?;
                }
            }

            next_in = p.0;
            unistd::close(p.1).map_err(error_to_string)?;
        }
        unistd::close(next_in).map_err(error_to_string)?;
        Ok(())
    }
}

impl RedirectIn {
    pub fn extract(self, shell: &mut Shell) -> Result<io::RawFd, String> {
        match self {
            RedirectIn::Normal(filepath) => {
                open_file(filepath.extract(shell)?.as_str(), fcntl::OFlag::O_RDONLY)
            }
            // TODO ヒアドキュメント実装
            RedirectIn::Heredoc(end_of_string) => unimplemented!(),
        }
    }
}
impl RedirectOut {
    pub fn extract(self, shell: &mut Shell) -> Result<io::RawFd, String> {
        match self {
            RedirectOut::Normal(filepath) => open_file(
                filepath.extract(shell)?.as_str(),
                fcntl::OFlag::O_CREAT | fcntl::OFlag::O_WRONLY | fcntl::OFlag::O_EXCL,
            ),
            RedirectOut::Overwrite(filepath) => open_file(
                &filepath.extract(shell)?.as_str(),
                fcntl::OFlag::O_CREAT | fcntl::OFlag::O_WRONLY | fcntl::OFlag::O_TRUNC,
            ),
            RedirectOut::Append(filepath) => open_file(
                &filepath.extract(shell)?.as_str(),
                fcntl::OFlag::O_CREAT | fcntl::OFlag::O_WRONLY | fcntl::OFlag::O_APPEND,
            ),
        }
    }
}

impl Command {
    pub fn exec(
        self,
        shell: &mut Shell,
        stdin: io::RawFd,
        stdout: io::RawFd,
        to_close: Vec<io::RawFd>,
    ) -> Result<(), String> {
        let stdin = if let Some(r) = self.redirect_in {
            r.extract(shell)?
        } else {
            stdin
        };
        let stdout = if let Some(r) = self.redirect_out {
            r.extract(shell)?
        } else {
            stdout
        };

        match self.exe {
            Executable::File {
                command_name,
                arguments,
            } => {
                // コマンドテーブルから引く
                fn command_search(shell: &mut Shell, name: &str) -> Result<CommandType, String> {
                    match shell.command_table.get(name) {
                        Some(cmd) => Ok(cmd.clone()),
                        None => {
                            // パスとして存在するか
                            if !std::path::Path::new(name).exists() {
                                return Err(error_to_string(format!("`{}` not found.", name)));
                            }
                            Ok(CommandType::External(PathBuf::from(name)))
                        }
                    }
                }

                let command_name = command_name.extract(shell)?;
                let cmd = command_search(shell, command_name.as_str())?;

                let mut arguments = arguments;
                let cmd = match cmd {
                    CommandType::Alias(alias) => {
                        let tmp: Vec<String> = alias.split(" ").map(str::to_string).collect();
                        if tmp.is_empty() {
                            return Err(error_to_string("empty command."));
                        }

                        // 引数を追加
                        let mut new_arg: Vec<_> =
                            tmp[1..].iter().map(|a| Str::Raw(a.clone())).collect();
                        new_arg.append(&mut arguments);
                        arguments = new_arg;

                        let com_name = tmp.first().unwrap();
                        match command_search(shell, com_name) {
                            Ok(CommandType::Alias(_)) => {
                                return Err(error_to_string(format!(
                                    "`{}` not found.",
                                    &command_name
                                )))
                            }
                            Ok(other) => other,
                            Err(e) => return Err(e),
                        }
                    }
                    _ => cmd,
                };

                match cmd {
                    CommandType::External(path) => {
                        match unistd::fork() {
                            Ok(unistd::ForkResult::Parent { child, .. }) => {}
                            Ok(unistd::ForkResult::Child) => {
                                for fd in to_close {
                                    unistd::close(fd).map_err(error_then_exit).unwrap();
                                }

                                unistd::dup2(stdin, 0).unwrap();
                                unistd::dup2(stdout, 1).unwrap();

                                let path = ffi::CString::new(path.to_str().unwrap()).unwrap();
                                let mut argv = Vec::with_capacity(1 + arguments.len());
                                argv.push(path.clone());
                                let mut arg_str = Vec::new();
                                for a in arguments {
                                    let tmp = a.extract(shell).map_err(error_then_exit).unwrap();
                                    let cs =
                                        ffi::CString::new(tmp).map_err(error_then_exit).unwrap();
                                    arg_str.push(cs);
                                }
                                argv.append(&mut arg_str);
                                // eprintln!("path: {:?}, in: {}, out: {}", path, stdin, stdout);

                                unistd::execv(&path, &argv).map_err(error_then_exit);
                            }
                            _ => return Err(error_to_string(format!("fork failed."))),
                        }
                    }
                    CommandType::Builtin(f) => {
                        let args = arguments
                            .iter()
                            .map(|a| a.clone().extract(shell).unwrap())
                            .collect::<Vec<_>>();

                        let old_stdin = unistd::dup(0).map_err(error_to_string)?;
                        let old_stdout = unistd::dup(1).map_err(error_to_string)?;

                        // eprintln!("builtin `{:?}`, in: {}, out: {}", command_name, stdin, stdout);

                        unistd::dup2(stdin, 0).map_err(error_to_string)?;
                        unistd::dup2(stdout, 1).map_err(error_to_string)?;

                        f(shell, args);

                        unistd::dup2(old_stdin, 0).map_err(error_to_string)?;
                        unistd::dup2(old_stdout, 1).map_err(error_to_string)?;

                        unistd::close(old_stdin).map_err(error_to_string)?;
                        unistd::close(old_stdout).map_err(error_to_string)?;
                    }
                    _ => {}
                }
            }
            Executable::SubShell(cmds) => {
                // 新しいシェルでcmdsを実行
                let mut child_shell = Shell::fromParent(&shell);
                match unistd::fork() {
                    Ok(unistd::ForkResult::Parent { child, .. }) => {}
                    Ok(unistd::ForkResult::Child) => {
                        for fd in to_close {
                            unistd::close(fd).map_err(error_then_exit).unwrap();
                        }


                        unistd::dup2(stdin, 0).map_err(error_then_exit);
                        unistd::dup2(stdout, 1).map_err(error_then_exit);
                        child_shell.exec(cmds).map_err(error_then_exit);
                        std::process::exit(0);
                    }
                    _ => return Err(error_to_string(format!("fork failed."))),
                }
            }
        }
        Ok(())
    }
}

impl Str {
    pub fn extract(self, shell: &mut Shell) -> Result<String, String> {
        match self {
            Str::Raw(s) => Ok(s.clone()),
            Str::Variable(v) => std::env::var(v).map_err(error_to_string),
            Str::SubShellResult(list) => {
                // pipeを作成して、in側をopenしてバッファに書き込む
                // これを文字列として返す
                let pfd = unistd::pipe().map_err(error_to_string)?;
                // eprintln!("pipe pair {} <-- {}", pfd.0, pfd.1);

                let mut child_shell = Shell::fromParent(&shell);
                match unistd::fork() {
                    Ok(unistd::ForkResult::Parent { child, .. }) => {
                        unistd::close(pfd.1).unwrap();
                        let mut buf = Vec::new();
                        let mut buf_size = 256;
                        loop {
                            let mut tmp = vec![0; buf_size];
                            let bytes = unistd::read(pfd.0, &mut tmp).unwrap();
                            if bytes == 0 {
                                break;
                            }
                            buf.append(&mut tmp);
                            buf_size <<= 1; // 次回は2倍の容量確保する
                        }

                        match sys::wait::waitpid(child, None) {
                            Ok(_) => {
                                let ret = String::from_utf8(buf)
                                    .unwrap()
                                    .trim_matches(char::from(0))
                                    .trim()
                                    .to_string();
                                Ok(ret)
                            }
                            Err(e) => Err(error_to_string(format!("{}", e))),
                        }
                    }
                    Ok(unistd::ForkResult::Child) => {
                        unistd::close(pfd.0).unwrap();
                        unistd::dup2(pfd.1, 1).map_err(error_then_exit);
                        let st = child_shell.exec(list).unwrap_or(1);
                        // eprintln!("child exited with {}", st);
                        std::process::exit(st);
                    }
                    _ => Err(error_to_string("fork failed")),
                }
            }
            Str::Quoted(cont) => {
                let mut ret = String::new();
                for c in cont {
                    ret.push_str(c.extract(shell)?.as_str());
                }
                Ok(ret)
            }
        }
    }
}

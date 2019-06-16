use nix::unistd;
use std::path;


use super::execute;
use super::structures;
// エイリアスを登録
pub fn alias(shell: &mut structures::Shell, mut argv: Vec<String>) -> execute::ExitCode {
    let usage = "usage: `$ alias name = content`";
    if argv.len() != 3 || argv[1].as_str() != "=" {
        eprintln!("alias: mismatch arguments. {}", usage);
        return 1;
    }
    let name = argv.remove(0);
    let _ = argv.remove(0);
    let body = argv.remove(0);
    shell
        .command_table
        .insert(name, structures::CommandType::Alias(body));
    0
}
// エイリアスを削除
pub fn unalias(shell: &mut structures::Shell, mut argv: Vec<String>) -> execute::ExitCode {
    let usage = "usage: `$ unalias name";
    if argv.len() != 1 {
        eprintln!("unalias: mismatch arguments. {}", usage);
        return 1;
    }
    let name = argv.remove(0);
    if shell.command_table.remove(&name).is_none() {
        eprintln!("unalias: `{}` is not registered.", &name);
        return 2;
    }
    0
}

// cd コマンド
// カレントディレクトリを変更
// 引数を指定しない場合は何もしない
// 2つ以上の引数を指定した場合、エラーメッセージを出して終了
pub fn cd(_: &mut structures::Shell, argv: Vec<String>) -> execute::ExitCode {
    if argv.len() == 0 {
        return 1;
    }
    if argv.len() > 1 {
        eprintln!("cd: too many arguments.");
        return 2;
    }
    match unistd::chdir(path::Path::new(&argv[0])) {
        Err(e) => {
            eprintln!("{}", e);
            1
        }
        _ => 0,
    }
}

// コマンド名の実体を調べる
pub fn type_(shell: &mut structures::Shell, argv: Vec<String>) -> execute::ExitCode {
    for name in argv {
        match shell.command_table.get(name.as_str()) {
            Some(cmd) => match cmd {
                structures::CommandType::Builtin(_) => {
                    println!("`{}` is a builtin function.", &name)
                }
                structures::CommandType::External(path) => println!("{}", path.display()),
                structures::CommandType::Alias(alias) => {
                    println!("`{}` is an alias of `{}`", &name, alias)
                }
            },
            None => {
                println!("`{}` not found.", &name);
            }
        };
    }
    0
}

// 組み込み関数
pub fn echo(_: &mut structures::Shell, argv: Vec<String>) -> execute::ExitCode {
    if argv.len() > 0 {
        eprintln!("too many arguments");
        return 1;
    }

    loop {
        let mut s = String::new();
        let bytes = std::io::stdin().read_line(&mut s).unwrap();
        if bytes == 0 {
            break;
        }
        print!("{}", s);
    }
    0
}

pub fn exit(_: &mut structures::Shell, argv: Vec<String>) -> execute::ExitCode {
    println!("good bye.");
    std::process::exit(0);
}

// 環境変数を設定
pub fn export(_: &mut structures::Shell, mut argv: Vec<String>) -> execute::ExitCode {
    let usage = "usage: `$ export env_var_name = content`";
    if argv.len() != 3 || argv[1].as_str() != "=" {
        eprintln!("export: mismatch arguments. {}", usage);
        return 1;
    }
    let name = argv.remove(0);
    let _ = argv.remove(0);
    let body = argv.remove(0);
    std::env::set_var(name, body);
    0
}
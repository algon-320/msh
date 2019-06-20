use nix::unistd;

use super::execute;
use super::structures;
use std::collections::HashMap;
use std::path;

/// aliasを追加
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
/// エイリアスを削除
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

/// cd コマンド
/// カレントディレクトリを変更
/// 引数を指定しない場合は $HOME が指すディレクトリに移動する
/// 2つ以上の引数を指定した場合、エラーメッセージを出して終了
/// 元のディレクトリを $OLDPWD に格納した上で新しいディレクトリに移動する
/// `cd -`のようにすると、$OLDPWD が指すディレクトリに移動する
pub fn cd(_: &mut structures::Shell, argv: Vec<String>) -> execute::ExitCode {
    if argv.len() > 1 {
        eprintln!("cd: too many arguments.");
        return 1;
    }

    let home_dir = std::env::var("HOME").unwrap();
    let oldpwd = std::env::var("OLDPWD");
    let new_wd = if argv.len() == 0 {
        path::Path::new(&home_dir)
    } else if argv.first().unwrap().as_str() == "-" && oldpwd.is_ok() {
        path::Path::new(oldpwd.as_ref().unwrap())
    } else {
        path::Path::new(&argv[0])
    };

    let pwd = std::env::current_dir().unwrap();
    let pwd = pwd.as_path().to_str().unwrap();
    std::env::set_var("OLDPWD", pwd);

    match unistd::chdir(new_wd) {
        Err(e) => {
            eprintln!("cd: {}", e);
            1
        }
        _ => 0,
    }
}

/// コマンド名の実体を調べる
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

/// 組み込み関数
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

/// シェルを終了する
pub fn exit(_: &mut structures::Shell, argv: Vec<String>) -> execute::ExitCode {
    println!("good bye.");
    std::process::exit(0);
}

/// 環境変数を設定
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

/// シェル変数を設定する
pub fn var(shell: &mut structures::Shell, mut argv: Vec<String>) -> execute::ExitCode {
    let usage = "usage: `$ var var_name = content`";
    if argv.len() != 3 || argv[1].as_str() != "=" {
        eprintln!("var: mismatch arguments. {}", usage);
        return 1;
    }
    let name = argv.remove(0);
    let _ = argv.remove(0);
    let body = argv.remove(0);
    shell.variables.insert(name, body);
    0
}

/// シェル変数・環境変数を削除する
pub fn unset(shell: &mut structures::Shell, mut argv: Vec<String>) -> execute::ExitCode {
    let usage = "usage: `$ unset var_name";
    if argv.len() != 1 {
        eprintln!("unset: mismatch arguments. {}", usage);
        return 1;
    }
    let name = argv.remove(0);
    std::env::remove_var(&name);
    shell.variables.remove(&name);
    0
}

/// command_tableを再設定する
pub fn reload_path(shell: &mut structures::Shell, _: Vec<String>) -> execute::ExitCode {
    let mut command_table: HashMap<String, structures::CommandType> = HashMap::new();
    // PATH からコマンドパスのテーブルを構築
    let path_str = match std::env::var("PATH") {
        Ok(s) => s,
        Err(_) => "".to_string(),
    };

    // println!("{}", path_str);
    for dir in path_str.split(":").map(|p| path::Path::new(p)) {
        if !dir.exists() {
            // eprintln!("path `{}` doesn't exist. skipped.", dir.display());
            continue;
        }
        if !dir.is_dir() {
            // eprintln!("`{}` is not a directory. skipped.", dir.display());
            continue;
        }

        for entry in dir.read_dir().unwrap() {
            if let Ok(ent) = entry {
                if let Some(name) = ent.path().file_name() {
                    command_table.insert(
                        name.to_str().unwrap().to_string(),
                        structures::CommandType::External(ent.path().clone()),
                    );
                }
            }
        }
    }

    // 組み込み関数の登録
    command_table.insert(
        format!("builtin-echo"),
        structures::CommandType::Builtin(echo),
    );

    command_table.insert(format!("type"), structures::CommandType::Builtin(type_));

    command_table.insert(format!("cd"), structures::CommandType::Builtin(cd));

    command_table.insert(format!("alias"), structures::CommandType::Builtin(alias));

    command_table.insert(
        format!("unalias"),
        structures::CommandType::Builtin(unalias),
    );

    command_table.insert(format!("exit"), structures::CommandType::Builtin(exit));

    command_table.insert(format!("export"), structures::CommandType::Builtin(export));
    command_table.insert(format!("var"), structures::CommandType::Builtin(var));
    command_table.insert(format!("unset"), structures::CommandType::Builtin(unset));

    command_table.insert(
        format!("reload-path"),
        structures::CommandType::Builtin(reload_path),
    );

    shell.command_table = command_table;

    0
}
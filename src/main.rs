mod builtin_commands;
mod execute;
mod structures;
mod msh_grammar {
    include!(concat!(env!("OUT_DIR"), "/msh_grammar.rs"));
}

use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::path;

fn main() {
    println!("msh version {}", env!("CARGO_PKG_VERSION"));

    let mut command_table: HashMap<String, structures::CommandType> = HashMap::new();
    {
        // PATH からコマンドパスのテーブルを構築
        let path_str = env!("PATH");
        // println!("{}", path_str);
        for dir in path_str.split(":").map(|p| path::Path::new(p)) {
            if !dir.exists() {
                eprintln!("path `{}` doesn't exist. skipped.", dir.display());
                continue;
            }
            if !dir.is_dir() {
                eprintln!("`{}` is not a directory. skipped.", dir.display());
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
            structures::CommandType::Builtin(builtin_commands::echo),
        );
        command_table.insert(
            format!("type"),
            structures::CommandType::Builtin(builtin_commands::type_),
        );
        command_table.insert(
            format!("cd"),
            structures::CommandType::Builtin(builtin_commands::cd),
        );

        command_table.insert(
            format!("alias"),
            structures::CommandType::Builtin(builtin_commands::alias),
        );
        command_table.insert(
            format!("unalias"),
            structures::CommandType::Builtin(builtin_commands::unalias),
        );
        command_table.insert(
            format!("exit"),
            structures::CommandType::Builtin(builtin_commands::exit),
        );
    }

    let mut shell = structures::Shell::new(command_table);

    loop {
        print!("$ "); // prompt
        io::stdout().flush().unwrap();
        let mut s = String::new();
        let bytes = io::stdin().read_line(&mut s).unwrap();
        if bytes == 0 {
            break;
        }

        match msh_grammar::command_line(&s.trim()) {
            Ok(r) => match r {
                Some(list) => {
                    // println!("{}\n{}\n{}", "=".repeat(80), r.print(0), "=".repeat(80));
                    match shell.exec(list) {
                        Ok(exit_code) => println!("=> exit_status: {}", exit_code),
                        Err(e) => println!("error: {}", e),
                    }
                }
                None => {}
            },
            Err(e) => {
                println!("parsing error. {}", e);
            }
        };
    }
    builtin_commands::exit(&mut shell, Vec::new());
}

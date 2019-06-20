mod builtin_commands;
mod execute;
mod structures;
mod msh_grammar {
    include!(concat!(env!("OUT_DIR"), "/msh_grammar.rs"));
}

use std::io;
use std::io::Write;

fn main() {
    println!("msh version {}", env!("CARGO_PKG_VERSION"));
    let mut shell = structures::Shell::new();
    loop {
        let user = std::env::var("USER").unwrap_or(format!("(unknown)"));
        print!("{}:{}$ ", user, std::env::current_dir().unwrap().display()); // prompt
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

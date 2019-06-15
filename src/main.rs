mod structures;
use structures::*;

mod msh_grammar {
    include!(concat!(env!("OUT_DIR"), "/msh_grammar.rs"));
}

fn main() {
    let commands = "a ; b";
    match msh_grammar::command_line(commands) {
        Ok(r) => match r {
            Some(list) => println!("{}\n{}\n{}", "=".repeat(80), list.print(0), "=".repeat(80)),
            None => {}
        }
        Err(e) => {
            println!("parsing error. {}", e);
        }
    };
}

use std::collections::HashMap;
use std::collections::VecDeque;

use std::ffi::OsString;

use std::path::PathBuf;

use super::execute;

#[derive(Clone)]
pub enum CommandType {
    External(PathBuf),
    Builtin(fn(&mut Shell, Vec<String>) -> execute::ExitCode),
    Alias(String),
}

pub struct Shell<'a> {
    pub id: usize,
    pub parent: Option<&'a Shell<'a>>,
    pub command_table: HashMap<String, CommandType>,
}

#[derive(Debug, Clone)]
pub struct List(pub Vec<Connector>);

#[derive(Debug, Clone)]
pub enum Connector {
    Continue(PipeLine),
    ListTerm(PipeLine),
}

#[derive(Debug, Clone)]
pub struct PipeLine(pub VecDeque<Pipe>);

#[derive(Debug, Clone)]
pub enum Pipe {
    Stdout(Command),
    Both(Command),
    PipeLineTerm(Command),
}

#[derive(Debug, Clone)]
pub enum Executable {
    File {
        command_name: Str,
        arguments: Vec<Str>,
    },
    SubShell(List),
}

#[derive(Debug, Clone)]
pub struct Command {
    pub exe: Executable,
    pub redirect_in: Option<RedirectIn>,
    pub redirect_out: Option<RedirectOut>,
}

pub enum Redirect {
    In(RedirectIn),
    Out(RedirectOut),
}

#[derive(Debug, Clone)]
pub enum RedirectIn {
    Normal(Str),
    Heredoc(String),
}
#[derive(Debug, Clone)]
pub enum RedirectOut {
    Normal(Str),
    Overwrite(Str),
    Append(Str),
}

#[derive(Debug, Clone)]
pub enum Str {
    Raw(String),
    Variable(String),
    SubShellResult(List),
    Quoted(Vec<Str>),
}


static INDENT_WIDTH: usize = 2;
// デバッグ用関数
fn gen_indent(d: usize) -> String {
    " ".repeat(d)
}
impl List {
    pub fn print(&self, indent: usize) -> String {
        format!(
            "{}List[\n{}\n{}]",
            gen_indent(indent),
            self.0.iter().fold(String::new(), |s, c| {
                s + &c.print(indent + INDENT_WIDTH) + ",\n"
            }),
            gen_indent(indent)
        )
    }
}

impl Connector {
    pub fn print(&self, indent: usize) -> String {
        match self {
            Connector::Continue(p) => format!(
                "{}Continue(\n{},\n{})",
                gen_indent(indent),
                p.print(indent + INDENT_WIDTH),
                gen_indent(indent)
            ),
            Connector::ListTerm(p) => format!(
                "{}ListTerm(\n{},\n{})",
                gen_indent(indent),
                p.print(indent + INDENT_WIDTH),
                gen_indent(indent)
            ),
        }
    }
}

impl PipeLine {
    pub fn print(&self, indent: usize) -> String {
        format!(
            "{}PipeLine[\n{}\n{}]",
            gen_indent(indent),
            self.0.iter().fold(String::new(), |s, c| {
                s + &c.print(indent + INDENT_WIDTH) + ",\n"
            }),
            gen_indent(indent)
        )
    }
}

impl Pipe {
    pub fn print(&self, indent: usize) -> String {
        match self {
            Pipe::PipeLineTerm(c) => format!(
                "{}PipeLineTerm(\n{},\n{})",
                gen_indent(indent),
                c.print(indent + INDENT_WIDTH),
                gen_indent(indent)
            ),
            Pipe::Stdout(c) => format!(
                "{}Stdout(\n{},\n{})",
                gen_indent(indent),
                c.print(indent + INDENT_WIDTH),
                gen_indent(indent)
            ),
            Pipe::Both(c) => format!(
                "{}Both(\n{},\n{})",
                gen_indent(indent),
                c.print(indent + INDENT_WIDTH),
                gen_indent(indent)
            ),
        }
    }
}
impl Executable {
    pub fn print(&self, indent: usize) -> String {
        match self {
            Executable::File {
                command_name: name,
                arguments: arg,
            } => format!(
                "{}File {{ name: {:?}, argument: {:?} }}",
                gen_indent(indent),
                name,
                arg
            ),
            Executable::SubShell(cb) => format!(
                "{}SubShell(\n{}\n{})",
                gen_indent(indent),
                cb.print(indent + INDENT_WIDTH),
                gen_indent(indent)
            ),
        }
    }
}
impl Command {
    pub fn print(&self, indent: usize) -> String {
        format!(
            "{}Command {{\n{}exe:\n{}\n{}redirect_in: {:?}\n{}redirect_out: {:?}\n{}}}",
            gen_indent(indent),
            gen_indent(indent + INDENT_WIDTH),
            self.exe.print(indent + INDENT_WIDTH * 2),
            gen_indent(indent + INDENT_WIDTH),
            self.redirect_in,
            gen_indent(indent + INDENT_WIDTH),
            self.redirect_out,
            gen_indent(indent)
        )
    }
}

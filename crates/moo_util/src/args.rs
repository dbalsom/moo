use crate::display::args::{display_parser, DisplayParams};
use bpaf::{construct, long, pure, Parser};
use std::{
    fmt::{Display, Formatter},
    io::Write,
    path::PathBuf,
};

#[derive(Clone, Debug)]
pub(crate) enum Command {
    Version,
    Display(DisplayParams),
    //Dump(DumpParams),
    //Find(FindParams),
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Version => write!(f, "version"),
            Command::Display(_) => write!(f, "display"),
            //Command::Dump(_) => write!(f, "dump"),
            //Command::Find(_) => write!(f, "find"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AppParams {
    pub global:  GlobalOptions,
    pub command: Command,
}

#[derive(Debug)]
pub struct GlobalOptions {
    pub silent: bool,
}

impl GlobalOptions {
    pub fn loud<F: FnMut()>(&self, mut f: F) {
        if !self.silent {
            f();
            std::io::stdout().flush().unwrap();
        }
    }
}

pub fn global_options_parser() -> impl Parser<GlobalOptions> {
    let silent = long("silent")
        .help("Suppress all output except required output")
        .switch(); // Switch returns a bool, true if the flag is present

    construct!(GlobalOptions { silent })
}

pub(crate) fn in_file_parser() -> impl Parser<PathBuf> {
    long("in_file")
        .short('i')
        .argument::<PathBuf>("INPUT_FILE")
        .help("Path to input file")
}

pub(crate) fn command_parser() -> impl Parser<AppParams> {
    let global = global_options_parser();

    let version = pure(Command::Version)
        .to_options()
        .command("version")
        .help("Display version information and exit");

    let display = construct!(Command::Display(display_parser()))
        .to_options()
        .command("display")
        .help("Display a test in human-readable format");

    // let find = construct!(Command::Find(find_parser()))
    //     .to_options()
    //     .command("find")
    //     .help("Find a test given its hash");

    let command = construct!([version, display]);

    construct!(AppParams { global, command })
}

pub(crate) fn hash_parser() -> impl Parser<String> {
    long("hash")
        .short('h')
        .argument::<String>("HASH")
        .help("Hexadecimal hash of a test")
}

/*
    MOO-rs Copyright 2025 Daniel Balsom
    https://github.com/dbalsom/moo

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/
use std::{
    fmt::{Display, Formatter},
    io::Write,
    path::PathBuf,
};

use crate::commands::{
    check::args::CheckParams,
    display::args::{display_parser, DisplayParams},
    find::args::FindParams,
};

use crate::commands::{check::args::check_parser, find::args::find_parser};
use bpaf::{construct, long, pure, Parser};

#[derive(Clone, Debug)]
pub(crate) enum Command {
    Version,
    Display(DisplayParams),
    //Dump(DumpParams),
    Find(FindParams),
    Check(CheckParams),
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Version => write!(f, "version"),
            Command::Display(_) => write!(f, "display"),
            //Command::Dump(_) => write!(f, "dump"),
            Command::Find(_) => write!(f, "find"),
            Command::Check(_) => write!(f, "check"),
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

pub(crate) fn in_path_parser() -> impl Parser<PathBuf> {
    long("input")
        .argument::<PathBuf>("INPUT_PATH")
        .help("Path to input file or directory")
}

pub(crate) fn out_path_parser() -> impl Parser<PathBuf> {
    long("output")
        .argument::<PathBuf>("OUTPUT_PATH")
        .help("Path to output file or directory")
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

    let find = construct!(Command::Find(find_parser()))
        .to_options()
        .command("find")
        .help("Find a test given its hash");

    let check = construct!(Command::Check(check_parser()))
        .to_options()
        .command("check")
        .help("Check integrity of MOO test files");

    let command = construct!([version, display, find, check]);

    construct!(AppParams { global, command })
}

pub(crate) fn hash_parser() -> impl Parser<String> {
    long("hash")
        .short('h')
        .argument::<String>("HASH")
        .help("Hexadecimal hash of a test")
}

pub(crate) fn index_parser() -> impl Parser<usize> {
    long("index")
        .short('i')
        .argument::<usize>("INDEX")
        .help("Index of the test to display")
}

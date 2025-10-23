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
mod args;
mod check;
mod display;
mod file;
mod find;
mod util;
mod working_set;

use crate::args::{command_parser, Command};

use anyhow::Error;
use bpaf::Parser;

fn main() -> Result<(), Error> {
    env_logger::init();

    let app_params = command_parser().run();

    let command_result = match &app_params.command {
        Command::Version => {
            println!("mootility v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Display(params) => display::run(&app_params.global, params),
        Command::Find(params) => find::run(&app_params.global, params),
        Command::Check(params) => check::run(&app_params.global, params),
    };

    match command_result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Command '{}' failed: {}", app_params.command, e);
            for cause in e.chain().skip(1) {
                eprintln!("Caused by: {}", cause);
            }
            std::process::exit(1);
        }
    }
}

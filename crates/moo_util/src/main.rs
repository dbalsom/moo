mod args;
mod display;

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
    };

    Ok(())
}

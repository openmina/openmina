pub mod commands;
pub use commands::{Command, CommandError};

mod exit_with_error;
pub use exit_with_error::exit_with_error;

use structopt::StructOpt;

fn main() {
    match commands::Command::from_args().run() {
        Ok(_) => {}
        Err(err) => exit_with_error(err),
    }
}

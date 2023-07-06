pub mod commands;
use clap::Parser;
pub use commands::CommandError;

mod exit_with_error;
pub use exit_with_error::exit_with_error;

fn main() {
    match commands::OpenminaCli::parse().command.run() {
        Ok(_) => {}
        Err(err) => exit_with_error(err),
    }
}

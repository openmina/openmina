use console::style;
use std::fmt::Display;
use std::process;

pub fn exit_with_error<E: Display>(error: E) -> ! {
    eprintln!("{} {:#}", style("[ERROR]").red().bold(), error,);
    process::exit(1)
}

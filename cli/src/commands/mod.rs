pub mod misc;
pub mod snark;
pub mod snarker;

pub type CommandError = Box<dyn std::error::Error>;

#[derive(Debug, clap::Parser)]
#[command(name = "openmina", about = "Openmina Cli")]
pub struct OpenminaCli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Snark(snark::Snark),
    /// Standalone snarker
    Snarker(snarker::Snarker),
    /// Miscilaneous utilities.
    Misc(misc::Misc),
}

impl Command {
    pub fn run(self) -> Result<(), crate::CommandError> {
        match self {
            Self::Snark(v) => v.run(),
            Self::Snarker(v) => v.run(),
            Self::Misc(v) => v.run(),
        }
    }
}

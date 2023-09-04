pub mod misc;
pub mod node;
pub mod replay;
pub mod snark;

pub type CommandError = Box<dyn std::error::Error>;

#[derive(Debug, clap::Parser)]
#[command(name = "openmina", about = "Openmina Cli")]
pub struct OpenminaCli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Openmina node.
    Node(node::Node),
    Snark(snark::Snark),
    /// Miscilaneous utilities.
    Misc(misc::Misc),
    Replay(replay::Replay),
}

impl Command {
    pub fn run(self) -> Result<(), crate::CommandError> {
        match self {
            Self::Snark(v) => v.run(),
            Self::Node(v) => v.run(),
            Self::Misc(v) => v.run(),
            Self::Replay(v) => v.run(),
        }
    }
}

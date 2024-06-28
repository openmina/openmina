use clap::ValueEnum;

pub mod build_info;
pub mod misc;
pub mod node;
pub mod replay;
pub mod snark;

#[derive(Debug, clap::Parser)]
#[command(name = "openmina", about = "Openmina Cli")]
pub struct OpenminaCli {
    #[arg(
        global = true,
        long,
        value_enum,
        default_value_t = Network::Devnet,
        env = "OPENMINA_NETWORK"
    )]
    /// Select the network (devnet or mainnet)
    pub network: Network,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Network {
    Devnet,
    Mainnet,
}

#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Openmina node.
    Node(node::Node),
    Snark(snark::Snark),
    /// Miscilaneous utilities.
    Misc(misc::Misc),
    Replay(replay::Replay),
    BuildInfo(build_info::Command),
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Snark(v) => v.run(),
            Self::Node(v) => v.run(),
            Self::Misc(v) => v.run(),
            Self::Replay(v) => v.run(),
            Self::BuildInfo(v) => v.run(),
        }
    }
}

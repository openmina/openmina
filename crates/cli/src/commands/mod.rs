pub mod build_info;
pub mod internal;
pub mod misc;
pub mod node;
pub mod replay;
pub mod snark;
pub mod wallet;

#[derive(Debug, clap::Parser)]
#[command(name = "mina", about = "Mina Cli")]
pub struct MinaCli {
    #[arg(
        global = true,
        long,
        value_enum,
        default_value_t = Network::Devnet,
        env = "MINA_NETWORK"
    )]
    /// Select the network (devnet or mainnet)
    pub network: Network,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Network {
    Devnet,
    Mainnet,
}

#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// Mina node.
    Node(node::Node),
    Snark(snark::Snark),
    /// Miscilaneous utilities.
    Misc(misc::Misc),
    Replay(replay::Replay),
    BuildInfo(build_info::Command),
    /// Wallet operations for managing accounts and sending transactions.
    Wallet(wallet::Wallet),
    /// Internal utilities for debugging and introspection.
    Internal(internal::Internal),
}

impl Command {
    pub fn run(self, network: Network) -> anyhow::Result<()> {
        match self {
            Self::Snark(v) => v.run(),
            Self::Node(v) => v.run(),
            Self::Misc(v) => v.run(),
            Self::Replay(v) => v.run(),
            Self::BuildInfo(v) => v.run(),
            Self::Wallet(v) => v.run(network),
            Self::Internal(v) => v.run(),
        }
    }
}

pub mod address;
pub mod balance;
pub mod generate;
pub mod send;
pub mod status;

use super::Network;
use crate::exit_with_error;

#[derive(Debug, clap::Args)]
pub struct Wallet {
    #[command(subcommand)]
    pub command: WalletCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum WalletCommand {
    /// Get the address from an encrypted key file
    Address(address::Address),
    /// Get account balance via GraphQL
    Balance(balance::Balance),
    /// Generate a new encrypted key pair
    Generate(generate::Generate),
    /// Send a payment transaction
    Send(send::Send),
    /// Check transaction status
    Status(status::Status),
}

impl Wallet {
    pub fn run(self, network: Network) -> anyhow::Result<()> {
        let result = match self.command {
            WalletCommand::Address(cmd) => cmd.run(),
            WalletCommand::Balance(cmd) => cmd.run(),
            WalletCommand::Generate(cmd) => cmd.run(),
            WalletCommand::Send(cmd) => cmd.run(network),
            WalletCommand::Status(cmd) => cmd.run(),
        };

        // Handle errors without backtraces for wallet commands
        if let Err(err) = result {
            exit_with_error(err);
        }

        Ok(())
    }
}

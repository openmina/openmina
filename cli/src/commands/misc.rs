use libp2p_identity::PeerId;
use node::p2p::identity::SecretKey;
use openmina_node_account::AccountSecretKey;

use crate::CommandError;

#[derive(Debug, clap::Args)]
pub struct Misc {
    #[command(subcommand)]
    command: MiscCommand,
}

impl Misc {
    pub fn run(self) -> Result<(), CommandError> {
        match self.command {
            MiscCommand::P2PKeyPair(command) => command.run(),
            MiscCommand::MinaKeyPair(command) => command.run(),
        }
    }
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum MiscCommand {
    P2PKeyPair(P2PKeyPair),
    MinaKeyPair(MinaKeyPair),
}

#[derive(Debug, Clone, clap::Args)]
pub struct P2PKeyPair {
    #[arg(long, short = 's', env = "OPENMINA_P2P_SEC_KEY")]
    p2p_secret_key: Option<SecretKey>,
}

impl P2PKeyPair {
    pub fn run(self) -> Result<(), CommandError> {
        let secret_key = self.p2p_secret_key.unwrap_or_else(SecretKey::rand);
        let public_key = secret_key.public_key();
        let peer_id = public_key.peer_id();
        let libp2p_peer_id = PeerId::from(peer_id);
        println!("secret key: {secret_key}");
        println!("public key: {public_key}");
        println!("peer_id:    {peer_id}");
        println!("libp2p_id:  {libp2p_peer_id}");

        Ok(())
    }
}

#[derive(Debug, Clone, clap::Args)]
pub struct MinaKeyPair {
    #[arg(long, short = 's', env = "OPENMINA_SEC_KEY")]
    secret_key: Option<AccountSecretKey>,
}

impl MinaKeyPair {
    pub fn run(self) -> Result<(), CommandError> {
        let secret_key = self.secret_key.unwrap_or_else(AccountSecretKey::rand);
        let public_key = secret_key.public_key();
        println!("secret key: {secret_key}");
        println!("public key: {public_key}");

        Ok(())
    }
}

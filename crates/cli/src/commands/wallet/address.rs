use std::path::PathBuf;

use anyhow::{Context, Result};
use mina_node_account::AccountSecretKey;

#[derive(Debug, clap::Args)]
pub struct Address {
    /// Path to encrypted key file
    #[arg(long, env)]
    pub from: PathBuf,

    /// Password to decrypt the key
    #[arg(
        env = "MINA_PRIVKEY_PASS",
        default_value = "",
        help = "Password to decrypt the key (env: MINA_PRIVKEY_PASS)"
    )]
    pub password: String,
}

impl Address {
    pub fn run(self) -> Result<()> {
        // Load and decrypt the key
        let secret_key = AccountSecretKey::from_encrypted_file(&self.from, &self.password)
            .with_context(|| format!("Failed to decrypt key file: {}", self.from.display()))?;

        // Get the public key
        let public_key = secret_key.public_key();

        // Display the address
        println!("{}", public_key);

        Ok(())
    }
}

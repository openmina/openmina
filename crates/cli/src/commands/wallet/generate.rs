use mina_node_account::AccountSecretKey;
use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Generate {
    /// Path where the encrypted key file will be saved
    #[arg(long)]
    pub output: PathBuf,

    /// Password to encrypt the key
    #[arg(
        env = "MINA_PRIVKEY_PASS",
        default_value = "",
        help = "Password to encrypt the key (env: MINA_PRIVKEY_PASS)"
    )]
    pub password: String,
}

impl Generate {
    pub fn run(self) -> anyhow::Result<()> {
        // Check if password is provided
        if self.password.is_empty() {
            anyhow::bail!(
                "Password is required. Provide it via --password argument or MINA_PRIVKEY_PASS environment variable"
            );
        }

        // Check if output file already exists
        if self.output.exists() {
            anyhow::bail!("File already exists: {}", self.output.display());
        }

        // Generate a new random keypair
        let secret_key = AccountSecretKey::rand();
        let public_key = secret_key.public_key();

        // Save the encrypted key to file
        secret_key.to_encrypted_file(&self.output, &self.password)?;

        // Save the public key to a separate file
        let pubkey_path = format!("{}.pub", self.output.display());
        std::fs::write(&pubkey_path, public_key.to_string())?;

        println!("Generated new encrypted key:");
        println!("  Private key: {}", self.output.display());
        println!("  Public key:  {}", pubkey_path);
        println!("  Address:     {}", public_key);
        println!();
        println!("Keep your encrypted key file and password secure!");

        Ok(())
    }
}

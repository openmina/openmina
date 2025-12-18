use libp2p_identity::PeerId;
use mina_node::{account::AccountSecretKey, p2p::identity::SecretKey};
use std::{fs::File, io::Write};

#[derive(Debug, clap::Args)]
pub struct Misc {
    #[command(subcommand)]
    command: MiscCommand,
}

impl Misc {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            MiscCommand::MinaEncryptedKey(command) => command.run(),
            MiscCommand::MinaKeyPair(command) => command.run(),
            MiscCommand::P2PKeyPair(command) => command.run(),
        }
    }
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum MiscCommand {
    MinaEncryptedKey(MinaEncryptedKey),
    MinaKeyPair(MinaKeyPair),
    P2PKeyPair(P2PKeyPair),
}

#[derive(Debug, Clone, clap::Args)]
pub struct P2PKeyPair {
    #[arg(long, short = 's', env = "MINA_P2P_SEC_KEY")]
    p2p_secret_key: Option<SecretKey>,
}

impl P2PKeyPair {
    pub fn run(self) -> anyhow::Result<()> {
        let secret_key = self.p2p_secret_key.unwrap_or_else(SecretKey::rand);
        let public_key = secret_key.public_key();
        let peer_id = public_key.peer_id();
        let libp2p_peer_id = PeerId::try_from(peer_id)?;
        println!("secret key: {secret_key}");
        println!("public key: {public_key}");
        println!("peer_id:    {peer_id}");
        println!("libp2p_id:  {libp2p_peer_id}");

        Ok(())
    }
}

/// Generates an unencrypted Mina key pair.
///
/// Outputs the public and private keys in a human-readable format.
/// For encrypted key storage suitable for block production, use
/// the `mina-encrypted-key` command instead.
#[derive(Debug, Clone, Default, clap::Args)]
pub struct MinaKeyPair {
    #[arg(long, short = 's', env = "MINA_SEC_KEY")]
    secret_key: Option<AccountSecretKey>,
}

impl MinaKeyPair {
    pub fn run(self) -> anyhow::Result<()> {
        let private_key = self.secret_key.unwrap_or_else(AccountSecretKey::rand);
        let public_key = private_key.public_key();

        println!("secret key: {private_key}");
        println!("public key: {public_key}");

        Ok(())
    }
}

/// Generate a new Mina key pair and save it as an encrypted JSON file
///
/// This command generates a new random Mina key pair (or uses a provided secret key)
/// and saves it to an encrypted JSON file format compatible with key generation
/// from the OCaml implementation.
/// The encrypted file can be used as a producer key for block production.
///
/// This command replicates the secret box functionality from `src/lib/secret_box`
/// in the OCaml implementation, providing compatible encrypted key storage.
///
/// # Examples
///
/// Generate a new encrypted key with password:
/// ```bash
/// mina misc mina-encrypted-key --password mypassword --file producer-key
/// ```
///
/// Generate a new encrypted key using environment variable for password:
/// ```bash
/// MINA_PRIVKEY_PASS=mypassword mina misc mina-encrypted-key --file producer-key
/// ```
///
/// Use an existing secret key:
/// ```bash
/// mina misc mina-encrypted-key --secret-key EKE... --password mypassword
/// ```
#[derive(Debug, Clone, clap::Args)]
pub struct MinaEncryptedKey {
    /// Optional existing secret key to encrypt. If not provided, generates a
    /// new random key
    #[arg(long, short = 's', env = "MINA_ENC_KEY")]
    secret_key: Option<AccountSecretKey>,

    /// Password to encrypt the key file with. Can be provided via
    /// MINA_PRIVKEY_PASS environment variable
    #[arg(env = "MINA_PRIVKEY_PASS", default_value = "")]
    password: String,

    /// Output file path for the encrypted key (default: mina_encrypted_key.json)
    #[arg(long, short = 'f', default_value = "mina_encrypted_key.json")]
    file: String,
}

impl MinaEncryptedKey {
    /// Execute the mina-encrypted-key command
    ///
    /// Generates a new Mina key pair (or uses provided secret key) and saves it
    /// as an encrypted JSON file that can be used for block production.
    ///
    /// It will also save the public key to the filename suffixed with `.pub`.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - On successful key generation and file creation
    /// * `Err(anyhow::Error)` - If key encryption or file writing fails
    ///
    /// # Output
    ///
    /// Prints the secret key and public key to stdout, and creates an encrypted
    /// JSON file at the specified path.
    pub fn run(self) -> anyhow::Result<()> {
        let secret_key = self.secret_key.unwrap_or_else(AccountSecretKey::rand);
        let public_key = secret_key.public_key();

        // Save the public key to a separate file
        let public_key_file = format!("{}.pub", self.file);

        if File::open(&public_key_file).is_ok() {
            return Err(anyhow::anyhow!(
                "Public key file '{}' already exists. Please choose a different file name.",
                public_key_file
            ));
        }

        secret_key
            .to_encrypted_file(&self.file, &self.password)
            .map_err(|e| {
                anyhow::anyhow!("Failed to encrypt key: {} into path '{}'", e, self.file,)
            })?;
        // Write the public key to the file
        let mut public_key_file = File::create(public_key_file)
            .map_err(|e| anyhow::anyhow!("Failed to create public key file: {}", e))?;
        public_key_file
            .write_all(public_key.to_string().as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write public key: {}", e))?;

        println!("secret key: {secret_key}");
        println!("public key: {public_key}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_mina_encrypted_key_generates_random_key() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_key.json");
        let file_path_str = file_path.to_str().unwrap().to_string();

        let cmd = MinaEncryptedKey {
            secret_key: None,
            password: "test_password".to_string(),
            file: file_path_str.clone(),
        };

        let result = cmd.run();
        assert!(result.is_ok());

        // Verify the file was created
        assert!(file_path.exists());

        // Verify the file contains encrypted data (should be JSON)
        let file_content = fs::read_to_string(&file_path).unwrap();
        assert!(file_content.starts_with('{'));
        assert!(file_content.ends_with('}'));
    }

    #[test]
    fn test_mina_encrypted_key_with_provided_secret_key() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_key_provided.json");
        let file_path_str = file_path.to_str().unwrap().to_string();

        let secret_key = AccountSecretKey::rand();
        let expected_public_key = secret_key.public_key();

        let cmd = MinaEncryptedKey {
            secret_key: Some(secret_key),
            password: "test_password".to_string(),
            file: file_path_str.clone(),
        };

        let result = cmd.run();
        assert!(result.is_ok());

        // Verify the file was created
        assert!(file_path.exists());

        // Verify we can load the key back and it matches
        let loaded_key = AccountSecretKey::from_encrypted_file(&file_path_str, "test_password");
        assert!(loaded_key.is_ok());
        let loaded_key = loaded_key.unwrap();
        assert_eq!(loaded_key.public_key(), expected_public_key);
    }

    #[test]
    fn test_mina_encrypted_key_with_empty_password() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_key_no_pass.json");
        let file_path_str = file_path.to_str().unwrap().to_string();

        let cmd = MinaEncryptedKey {
            secret_key: None,
            password: "".to_string(),
            file: file_path_str.clone(),
        };

        let result = cmd.run();
        assert!(result.is_ok());

        // Verify the file was created
        assert!(file_path.exists());

        // Verify we can load the key back with empty password
        let loaded_key = AccountSecretKey::from_encrypted_file(&file_path_str, "");
        assert!(loaded_key.is_ok());
    }

    #[test]
    fn test_mina_encrypted_key_wrong_password_fails() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_key_wrong_pass.json");
        let file_path_str = file_path.to_str().unwrap().to_string();

        let cmd = MinaEncryptedKey {
            secret_key: None,
            password: "correct_password".to_string(),
            file: file_path_str.clone(),
        };

        let result = cmd.run();
        assert!(result.is_ok());

        // Verify loading with wrong password fails
        let loaded_key = AccountSecretKey::from_encrypted_file(&file_path_str, "wrong_password");
        assert!(loaded_key.is_err());
    }

    #[test]
    fn test_mina_encrypted_key_invalid_file_path_fails() {
        let cmd = MinaEncryptedKey {
            secret_key: None,
            password: "test_password".to_string(),
            file: "/invalid/path/that/does/not/exist/key.json".to_string(),
        };

        let result = cmd.run();
        assert!(result.is_err());
    }

    #[test]
    fn test_mina_encrypted_key_roundtrip_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_roundtrip.json");
        let file_path_str = file_path.to_str().unwrap().to_string();

        // Generate a key with our command
        let original_secret_key = AccountSecretKey::rand();
        let original_public_key = original_secret_key.public_key();
        let password = "roundtrip_test_password";

        let cmd = MinaEncryptedKey {
            secret_key: Some(original_secret_key.clone()),
            password: password.to_string(),
            file: file_path_str.clone(),
        };

        let result = cmd.run();
        assert!(result.is_ok());

        // Load the key back using the secret key methods directly
        let loaded_secret_key = AccountSecretKey::from_encrypted_file(&file_path_str, password);
        assert!(loaded_secret_key.is_ok());
        let loaded_secret_key = loaded_secret_key.unwrap();
        let loaded_public_key = loaded_secret_key.public_key();

        // Verify the keys match exactly
        assert_eq!(original_public_key, loaded_public_key);
        assert_eq!(
            original_secret_key.to_string(),
            loaded_secret_key.to_string()
        );
    }

    #[test]
    fn test_mina_key_pair_generates_random_key() {
        let cmd = MinaKeyPair::default();

        let result = cmd.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_mina_key_pair_with_provided_secret_key() {
        let secret_key = AccountSecretKey::rand();
        let cmd = MinaKeyPair {
            secret_key: Some(secret_key),
        };

        let result = cmd.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_p2p_key_pair_generates_random_key() {
        let cmd = P2PKeyPair {
            p2p_secret_key: None,
        };

        let result = cmd.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_p2p_key_pair_with_provided_secret_key() {
        let secret_key = SecretKey::rand();
        let cmd = P2PKeyPair {
            p2p_secret_key: Some(secret_key),
        };

        let result = cmd.run();
        assert!(result.is_ok());
    }
}

//! Account management for Mina nodes
//!
//! This crate provides a high-level interface for managing Mina accounts,
//! built on top of the
//! [`mina-signer`](https://github.com/o1-labs/proof-systems/tree/master/signer)
//! crate. It handles cryptographic key generation, encryption/decryption of
//! secret keys, and address handling.
//!
//! # Overview
//!
//! The crate exports two main types:
//! - [`AccountSecretKey`] - Represents a private key that can be used to
//!   sign transactions
//! - [`AccountPublicKey`] - Represents a public key/address for receiving
//!   transactions
//!
//! # Key Features
//!
//! - **Key Generation**: Generate new random keypairs for Mina accounts
//! - **Key Encryption**: Encrypt and decrypt secret keys using password-based
//!   encryption
//! - **Address Format**: Encode and decode Mina addresses using the standard
//!   Base58Check format
//! - **Key Import/Export**: Read and write encrypted keys from/to files
//!
//! # Example Usage
//!
//! ```
//! use mina_node_account::{AccountSecretKey, AccountPublicKey};
//! use std::env;
//!
//! // Generate a new keypair
//! let secret_key = AccountSecretKey::rand();
//! let public_key = secret_key.public_key();
//!
//! // Save encrypted key to file in temp directory
//! let temp_dir = env::temp_dir();
//! let path = temp_dir.join(format!("test-wallet-{}", public_key));
//! let password = "secure-password";
//! secret_key.to_encrypted_file(&path, password)
//!     .expect("Failed to save key");
//!
//! // Load encrypted key from file
//! let loaded_key = AccountSecretKey::from_encrypted_file(&path, password)
//!     .expect("Failed to load key");
//!
//! // Get the public address
//! let address = AccountPublicKey::from(loaded_key.public_key());
//! println!("Address: {}", address);
//!
//! // Verify the keys match
//! assert_eq!(secret_key.public_key().to_string(),
//!            loaded_key.public_key().to_string());
//!
//! // Clean up
//! std::fs::remove_file(&path).ok();
//! ```
//!
//! # Cryptography
//!
//! Mina uses the Pasta curves (Pallas and Vesta) for its cryptographic
//! operations. These curves are specifically designed for efficient
//! recursive zero-knowledge proof composition.

mod public_key;
mod secret_key;
pub use public_key::AccountPublicKey;
pub use secret_key::AccountSecretKey;

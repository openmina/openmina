//! WebRTC connection authentication.
//!
//! This module provides cryptographic authentication for WebRTC connections
//! using SDP hashes and public key encryption to prevent man-in-the-middle
//! attacks. The authentication mechanism ensures that WebRTC connections are
//! established only between legitimate peers with verified identities.
//!
//! ## Security Model
//!
//! The connection authentication process works by:
//!
//! 1. **SDP Hash Combination**: Combining the SDP hashes from both the WebRTC
//!    offer and answer to create a unique authentication token
//! 2. **Public Key Encryption**: Encrypting the authentication data using the
//!    recipient's public key to ensure only they can decrypt it
//! 3. **Mutual Verification**: Both parties verify each other's ability to
//!    decrypt the authentication data, proving they possess the correct private
//!    keys
//!
//! ## Authentication Flow
//!
//! ```text
//! Peer A                                    Peer B
//!   |                                         |
//!   |  1. Create Offer (with SDP)            |
//!   |------------------------------------>   |
//!   |                                         |
//!   |  2. Create Answer (with SDP)           |
//!   |   <------------------------------------|
//!   |                                         |
//!   |  3. Generate ConnectionAuth from       |
//!   |     both SDP hashes                    |
//!   |                                         |
//!   |  4. Encrypt with peer's public key     |
//!   |------------------------------------>   |
//!   |                                         |
//!   |  5. Decrypt and verify                 |
//!   |   <------------------------------------|
//!   |                                         |
//!   |  6. Connection authenticated ✓         |
//! ```
//!
//! ## Security Properties
//!
//! - **Identity Verification**: Ensures both parties possess the private keys
//!   corresponding to their advertised public keys
//! - **Man-in-the-Middle Protection**: Prevents attackers from intercepting and
//!   modifying the connection establishment process
//! - **Replay Attack Prevention**: Uses unique SDP hashes for each connection
//!   attempt, preventing replay attacks

use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use crate::identity::{PublicKey, SecretKey};

use super::{Answer, Offer};

/// Connection authentication data derived from WebRTC signaling.
///
/// `ConnectionAuth` contains the authentication material generated from the
/// SDP (Session Description Protocol) hashes of both the WebRTC offer and
/// answer.
/// This creates a unique, connection-specific authentication token that can be
/// used to verify the authenticity of the WebRTC connection.
///
/// ## Construction
///
/// The authentication data is created by concatenating the SDP hashes from both
/// the offer and answer messages:
///
/// ```text
/// ConnectionAuth = SDP_Hash(Offer) || SDP_Hash(Answer)
/// ```
///
/// This ensures that both parties contributed to the authentication material
/// and that any tampering with either the offer or answer would be detected.
///
/// ## Security Properties
///
/// - **Uniqueness**: Each connection attempt generates unique SDP data,
///   preventing replay attacks
/// - **Integrity**: Any modification to the offer or answer changes the hashes,
///   invalidating the authentication
/// - **Binding**: Cryptographically binds the authentication to the specific
///   WebRTC session parameters
///
/// ## Usage
///
/// ```rust
/// use mina_p2p::webrtc::{ConnectionAuth, Offer, Answer};
///
/// let connection_auth = ConnectionAuth::new(&offer, &answer);
/// let encrypted_auth = connection_auth.encrypt(&my_secret_key, &peer_public_key, rng)?;
/// ```
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ConnectionAuth(Vec<u8>);

/// Encrypted connection authentication data.
///
/// `ConnectionAuthEncrypted` represents the connection authentication data after
/// it has been encrypted using public key cryptography. The encrypted data is
/// stored in a fixed-size array of 92 bytes, which corresponds to the output
/// size of the encryption algorithm used.
///
/// ## Encryption Process
///
/// The encryption uses the recipient's public key to ensure that only the
/// intended recipient can decrypt and verify the authentication data. This
/// prevents man-in-the-middle attackers from forging authentication tokens.
///
/// ## Fixed Size
///
/// The 92-byte fixed size is determined by the cryptographic parameters:
/// - The encryption algorithm produces a deterministic output size
/// - Fixed sizing enables efficient serialization and network transmission
/// - Prevents information leakage through size analysis
///
/// ## Network Transmission
///
/// This type is designed for transmission over the network and includes
/// serialization support for JSON and binary formats.
///
/// ## Example
///
/// ```rust
/// // After receiving encrypted authentication data
/// let decrypted_auth = encrypted_auth.decrypt(&my_secret_key, &peer_public_key)?;
/// // Verify that the decrypted data matches expected values
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionAuthEncrypted(Box<[u8; 92]>);

impl ConnectionAuth {
    /// Creates new connection authentication data from WebRTC offer and answer.
    ///
    /// This method generates connection authentication data by concatenating the
    /// SDP hashes from both the WebRTC offer and answer messages. The resulting
    /// authentication token is unique to this specific connection attempt and
    /// binds the authentication to the exact WebRTC session parameters.
    ///
    /// # Parameters
    ///
    /// * `offer` - The WebRTC offer containing SDP data and peer information
    /// * `answer` - The WebRTC answer containing SDP data and peer information
    ///
    /// # Returns
    ///
    /// A new `ConnectionAuth` instance containing the concatenated SDP hashes.
    ///
    /// # Security Considerations
    ///
    /// The authentication data is derived from both the offer and answer, ensuring
    /// that any tampering with either message will result in different authentication
    /// data. This prevents attackers from modifying signaling messages without
    /// detection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mina_p2p::webrtc::ConnectionAuth;
    ///
    /// let auth = ConnectionAuth::new(&offer, &answer);
    /// // Use auth for connection verification
    /// ```
    pub fn new(offer: &Offer, answer: &Answer) -> Self {
        Self([offer.sdp_hash(), answer.sdp_hash()].concat())
    }

    /// Encrypts the connection authentication data using public key cryptography.
    ///
    /// This method encrypts the authentication data using the recipient's
    /// public key, ensuring that only the intended recipient (who possesses the
    /// corresponding private key) can decrypt and verify the authentication
    /// token.
    ///
    /// # Parameters
    ///
    /// * `sec_key` - The sender's secret key used for encryption
    /// * `other_pk` - The recipient's public key used for encryption
    /// * `rng` - A cryptographically secure random number generator
    ///
    /// # Returns
    ///
    /// * `Some(ConnectionAuthEncrypted)` if encryption succeeds
    /// * `None` if encryption fails (e.g., due to invalid keys or cryptographic
    ///   errors)
    ///
    /// # Security Properties
    ///
    /// - **Confidentiality**: Only the holder of the corresponding private key can
    ///   decrypt the authentication data
    /// - **Authenticity**: The encryption process provides assurance about the
    ///   sender's identity
    ///
    /// # Example
    ///
    /// ```rust
    /// use rand::thread_rng;
    ///
    /// let mut rng = thread_rng();
    /// let encrypted_auth = connection_auth.encrypt(&my_secret_key, &peer_public_key, &mut rng);
    ///
    /// if let Some(encrypted) = encrypted_auth {
    ///     // Send encrypted authentication data to peer
    /// }
    /// ```
    pub fn encrypt(
        &self,
        sec_key: &SecretKey,
        other_pk: &PublicKey,
        rng: impl Rng + CryptoRng,
    ) -> Option<ConnectionAuthEncrypted> {
        let bytes = sec_key.encrypt_raw(other_pk, rng, &self.0).ok()?;
        bytes.try_into().ok()
    }
}

impl ConnectionAuthEncrypted {
    /// Decrypts the connection authentication data using public key cryptography.
    ///
    /// This method decrypts the authentication data using the recipient's
    /// secret key and the sender's public key. Successful decryption proves
    /// that the sender possesses the private key corresponding to their
    /// advertised public key, providing authentication and preventing
    /// man-in-the-middle attacks.
    ///
    /// # Parameters
    ///
    /// * `sec_key` - The recipient's secret key used for decryption
    /// * `other_pk` - The sender's public key used for decryption
    ///
    /// # Returns
    ///
    /// * `Some(ConnectionAuth)` if decryption succeeds and authentication is valid
    /// * `None` if decryption fails (e.g., due to invalid keys, corrupted data, or
    ///   cryptographic errors)
    ///
    /// # Security Verification
    ///
    /// Successful decryption provides several security guarantees:
    ///
    /// - **Identity Verification**: The sender possesses the private key
    ///   corresponding to their public key
    /// - **Message Integrity**: The encrypted data has not been tampered with
    /// - **Authenticity**: The authentication data came from the claimed sender
    ///
    /// # Usage in Authentication Flow
    ///
    /// This method is typically called during the final stage of WebRTC connection
    /// establishment to verify the peer's identity before allowing the connection
    /// to proceed.
    ///
    /// # Example
    ///
    /// ```rust
    /// // After receiving encrypted authentication data from peer
    /// if let Some(decrypted_auth) = encrypted_auth.decrypt(&my_secret_key, &peer_public_key) {
    ///     // Authentication successful, proceed with connection
    ///     println!("Peer authentication verified");
    /// } else {
    ///     // Authentication failed, reject connection
    ///     println!("Peer authentication failed");
    /// }
    /// ```
    pub fn decrypt(&self, sec_key: &SecretKey, other_pk: &PublicKey) -> Option<ConnectionAuth> {
        sec_key
            .decrypt_raw(other_pk, &*self.0)
            .map(ConnectionAuth)
            .ok()
    }
}

impl Serialize for ConnectionAuthEncrypted {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_vec().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ConnectionAuthEncrypted {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::deserialize(deserializer).and_then(|v| {
            use serde::de::Error;
            v.try_into().map_err(Error::custom)
        })
    }
}

impl TryFrom<Vec<u8>> for ConnectionAuthEncrypted {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl TryFrom<&[u8]> for ConnectionAuthEncrypted {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        value
            .try_into()
            .map(|v| Self(Box::new(v)))
            .map_err(|_| "ConnectionAuthEncrypted not in expected size")
    }
}

impl AsRef<[u8]> for ConnectionAuthEncrypted {
    fn as_ref(&self) -> &[u8] {
        &*self.0
    }
}

use std::{fmt, path::Path, str::FromStr};

use base64::Engine;
use ed25519_dalek::SigningKey as Ed25519SecretKey;
use openmina_core::{EncryptedSecretKey, EncryptedSecretKeyFile, EncryptionError};
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use crate::identity::PublicKey;

#[derive(Clone)]
pub struct SecretKey(Ed25519SecretKey);

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SecretKey").field(&"***").finish()
    }
}

impl SecretKey {
    const BASE58_CHECK_VERSION: u8 = 0x80;

    pub fn rand() -> Self {
        Self::rand_with(&mut rand::thread_rng())
    }

    pub fn rand_with(mut rng: impl Rng) -> Self {
        Self::from_bytes(rng.gen())
    }

    pub fn deterministic(i: usize) -> Self {
        let mut bytes = [0; 32];
        let bytes_len = bytes.len();
        let i_bytes = (i + 1).to_be_bytes();
        let i = bytes_len - i_bytes.len();
        bytes[i..bytes_len].copy_from_slice(&i_bytes);
        Self::from_bytes(bytes)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Ed25519SecretKey::from_bytes(&bytes))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }

    pub fn to_x25519(&self) -> x25519_dalek::StaticSecret {
        self.0.to_scalar_bytes().into()
    }

    pub fn from_encrypted_file(
        path: impl AsRef<Path>,
        password: &str,
    ) -> Result<Self, EncryptionError> {
        let encrypted = EncryptedSecretKeyFile::new(path)?;
        let decrypted = Self::try_decrypt(&encrypted, password)?;

        let keypair_string = String::from_utf8(decrypted.to_vec())
            .map_err(|e| EncryptionError::Other(e.to_string()))?;

        let parts: Vec<&str> = keypair_string.split(',').collect();

        if parts.len() != 3 {
            return Err(EncryptionError::Other(
                "libp2p keypair string must have 3 parts".to_string(),
            ));
        }

        let (secret_key_base64, _public_key_base64, _peer_id) = (parts[0], parts[1], parts[2]);

        let key_bytes = base64::engine::general_purpose::STANDARD
            .decode(secret_key_base64.as_bytes())
            .map_err(|e| EncryptionError::Other(e.to_string()))?;

        let key_bytes = key_bytes[4..36]
            .try_into()
            .map_err(|_| EncryptionError::Other("Invalid secret key length".to_string()))?;
        Ok(Self::from_bytes(key_bytes))
    }

    pub fn to_encrypted_file(
        &self,
        _password: &str,
        _path: impl AsRef<Path>,
    ) -> Result<(), EncryptionError> {
        todo!()
    }
}

impl EncryptedSecretKey for SecretKey {}

use aes_gcm::{
    aead::{Aead, AeadCore},
    Aes256Gcm, KeyInit,
};

// TODO: provide more detailed errors
#[derive(Debug, Clone)]
pub struct EncryptError();

impl std::fmt::Display for EncryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encryption error occurred")
    }
}

impl std::error::Error for EncryptError {}

impl SecretKey {
    fn shared_key(&self, other_pk: &PublicKey) -> Result<Aes256Gcm, EncryptError> {
        let key = self.to_x25519().diffie_hellman(&other_pk.to_x25519());
        if !key.was_contributory() {
            return Err(EncryptError());
        }
        let key = key.to_bytes();
        // eprintln!("[shared_key] {} & {} = {}", self.public_key(), other_pk, hex::encode(&key));
        let key: &aes_gcm::Key<Aes256Gcm> = (&key).into();
        Ok(Aes256Gcm::new(key))
    }

    pub fn encrypt_raw(
        &self,
        other_pk: &PublicKey,
        rng: impl Rng + CryptoRng,
        data: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let shared_key = self.shared_key(other_pk)?;
        let nonce = Aes256Gcm::generate_nonce(rng);
        let mut buffer = Vec::from(AsRef::<[u8]>::as_ref(&nonce));
        buffer.extend(
            shared_key
                .encrypt(&nonce, data)
                .or(Err(Box::new(EncryptError())))?,
        );
        Ok(buffer)
    }

    pub fn encrypt<M: EncryptableType>(
        &self,
        other_pk: &PublicKey,
        rng: impl Rng + CryptoRng,
        data: &M,
    ) -> Result<M::Encrypted, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(data).map_err(|_| EncryptError())?;
        self.encrypt_raw(other_pk, rng, &data).map(Into::into)
    }

    pub fn decrypt_raw(
        &self,
        other_pk: &PublicKey,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, EncryptError> {
        let shared_key = self.shared_key(other_pk)?;
        let (nonce, ciphertext) = ciphertext.split_at_checked(12).ok_or(EncryptError())?;
        shared_key
            .decrypt(nonce.into(), ciphertext)
            .or(Err(EncryptError()))
    }

    pub fn decrypt<M: EncryptableType>(
        &self,
        other_pk: &PublicKey,
        ciphertext: &M::Encrypted,
    ) -> Result<M, Box<dyn std::error::Error>> {
        let data: Vec<u8> = self.decrypt_raw(other_pk, ciphertext.as_ref())?;
        serde_json::from_slice(&data).map_err(Box::<dyn std::error::Error>::from)
    }
}

pub trait EncryptableType: Serialize + for<'a> Deserialize<'a> {
    type Encrypted: From<Vec<u8>> + AsRef<[u8]>;
}

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = bs58::encode(&self.to_bytes())
            .with_check_version(Self::BASE58_CHECK_VERSION)
            .into_string();
        write!(f, "{}", s)
    }
}

#[derive(thiserror::Error, Serialize, Deserialize, Debug, Clone)]
pub enum SecretKeyFromStrError {
    #[error("Base58 decode error: {0}")]
    Bs58(String),
}

impl FromStr for SecretKey {
    type Err = SecretKeyFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; 37];
        let size = bs58::decode(s)
            .with_check(Some(Self::BASE58_CHECK_VERSION))
            .into(&mut bytes)
            .map_err(|err| SecretKeyFromStrError::Bs58(err.to_string()))?;
        if size != 33 {
            return Err(SecretKeyFromStrError::Bs58(
                bs58::decode::Error::BufferTooSmall.to_string(),
            ));
        }
        Ok(Self::from_bytes(
            bytes[1..33].try_into().expect("Size checked above"),
        ))
    }
}

#[cfg(feature = "p2p-libp2p")]
impl TryFrom<SecretKey> for libp2p_identity::Keypair {
    type Error = libp2p_identity::DecodingError;

    fn try_from(value: SecretKey) -> Result<Self, Self::Error> {
        Self::ed25519_from_bytes(value.to_bytes())
    }
}

#[cfg(feature = "p2p-libp2p")]
impl TryFrom<libp2p_identity::Keypair> for SecretKey {
    type Error = ();

    fn try_from(value: libp2p_identity::Keypair) -> Result<Self, Self::Error> {
        let bytes = value.try_into_ed25519().or(Err(()))?.to_bytes();
        Ok(Self::from_bytes(bytes[0..32].try_into().or(Err(()))?))
    }
}

impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let b58: String = Deserialize::deserialize(deserializer)?;
        b58.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::SecretKey;

    #[test]
    fn secret_key_to_string_roundtrip() {
        let s = "5K9G39rCgREFMCk7S739JZprwpMsiLRQXcELErUSwhHwfdVR8HT";
        let sk = s.parse::<SecretKey>().expect("should be parseable");
        let unparsed = sk.to_string();
        assert_eq!(s, &unparsed);
    }

    #[test]
    fn test_libp2p_key_decrypt() {
        let password = "total-secure-pass";
        let key_path = "../tests/files/accounts/libp2p-key";

        let expected_peer_id = "12D3KooWDxyuJKSsVEwNR13UVwf4PEfs4yHkk3ecZipBPv3Y3Sac";

        let decrypted = SecretKey::from_encrypted_file(key_path, password)
            .expect("Failed to decrypt secret key file");

        let peer_id = decrypted.public_key().peer_id().to_libp2p_string();
        assert_eq!(expected_peer_id, peer_id);
    }
}

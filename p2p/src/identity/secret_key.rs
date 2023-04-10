use ed25519_dalek::SigningKey as Ed25519SecretKey;

use crate::identity::PublicKey;

#[derive(Clone)]
pub struct SecretKey(Ed25519SecretKey);

impl SecretKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Ed25519SecretKey::from_bytes(&bytes))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }
}

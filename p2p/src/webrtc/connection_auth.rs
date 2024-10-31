use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use crate::identity::{PublicKey, SecretKey};

use super::{Answer, Offer};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ConnectionAuth(Vec<u8>);

#[derive(Debug, Clone)]
pub struct ConnectionAuthEncrypted(Box<[u8; 92]>);

impl ConnectionAuth {
    pub fn new(offer: &Offer, answer: &Answer) -> Self {
        Self([offer.sdp_hash(), answer.sdp_hash()].concat())
    }

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

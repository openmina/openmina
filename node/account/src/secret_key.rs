use std::{fmt, fs, io, path::Path, str::FromStr};

use mina_p2p_messages::{bigint::BigInt, v2::SignatureLibPrivateKeyStableV1};
use mina_signer::seckey::SecKeyError;
use mina_signer::{keypair::KeypairError, CompressedPubKey, Keypair};
use openmina_core::constants::GENESIS_PRODUCER_SK;
use openmina_core::{EncryptedSecretKey, EncryptedSecretKeyFile, EncryptionError};
use rand::{rngs::StdRng, CryptoRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use super::AccountPublicKey;

#[derive(Clone)]
pub struct AccountSecretKey(Keypair);

impl std::fmt::Debug for AccountSecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AccountSecretKey").field(&"***").finish()
    }
}

lazy_static::lazy_static! {
    // TODO(binier): better way.
    static ref GENERATED_DETERMINISTIC: Vec<AccountSecretKey> = {
        let mut rng = StdRng::seed_from_u64(0);
        (0..10000)
            .map(|_| AccountSecretKey::rand_with(&mut rng))
            .collect()
    };
}

impl AccountSecretKey {
    const BASE58_CHECK_VERSION: u8 = 90;

    pub fn genesis_producer() -> Self {
        Self::from_str(GENESIS_PRODUCER_SK).unwrap()
    }

    pub fn deterministic(i: u64) -> Self {
        GENERATED_DETERMINISTIC[i as usize].clone()
    }

    pub fn deterministic_iter() -> impl Iterator<Item = &'static AccountSecretKey> {
        GENERATED_DETERMINISTIC.iter()
    }

    pub fn max_deterministic_count() -> usize {
        GENERATED_DETERMINISTIC.len()
    }

    pub fn rand() -> Self {
        Self::rand_with(rand::thread_rng())
    }

    pub fn rand_with(mut rng: impl Rng + CryptoRng) -> Self {
        Self(Keypair::rand(&mut rng).unwrap())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeypairError> {
        let mut bytes: [u8; 32] = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(KeypairError::SecretKey(SecKeyError::SecretKeyLength)),
        };

        // For some reason, `mina_signer::SecKey::from_bytes` reverse the bytes
        bytes.reverse();

        Ok(Self(Keypair::from_bytes(&bytes[..])?))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        // TODO(binier): refactor
        let mut bytes = hex::decode(self.0.to_hex()).unwrap();
        bytes.reverse();
        bytes.try_into().unwrap()
    }

    pub fn public_key(&self) -> AccountPublicKey {
        self.0.public.clone().into()
    }

    pub fn public_key_compressed(&self) -> CompressedPubKey {
        self.0.public.clone().into_compressed()
    }

    pub fn from_encrypted_file(
        path: impl AsRef<Path>,
        password: &str,
    ) -> Result<Self, EncryptionError> {
        Self::from_encrypted_reader(fs::File::open(path)?, password)
    }

    pub fn from_encrypted_reader(
        reader: impl io::Read,
        password: &str,
    ) -> Result<Self, EncryptionError> {
        let encrypted: EncryptedSecretKeyFile = serde_json::from_reader(reader)?;
        Self::from_encrypted(&encrypted, password)
    }

    pub fn from_encrypted(
        encrypted: &EncryptedSecretKeyFile,
        password: &str,
    ) -> Result<Self, EncryptionError> {
        let decrypted: Vec<u8> = Self::try_decrypt(encrypted, password)?;
        AccountSecretKey::from_bytes(&decrypted[1..])
            .map_err(|err| EncryptionError::Other(err.to_string()))
    }

    pub fn to_encrypted_file(
        &self,
        path: impl AsRef<Path>,
        password: &str,
    ) -> Result<(), EncryptionError> {
        if path.as_ref().exists() {
            panic!("File {} already exists", path.as_ref().display())
        }

        let f = fs::File::create(path)?;
        let encrypted = Self::try_encrypt(&self.to_bytes(), password)?;

        serde_json::to_writer(f, &encrypted)?;
        Ok(())
    }
}

impl EncryptedSecretKey for AccountSecretKey {}

impl From<AccountSecretKey> for Keypair {
    fn from(value: AccountSecretKey) -> Self {
        value.0
    }
}

impl From<AccountSecretKey> for SignatureLibPrivateKeyStableV1 {
    fn from(value: AccountSecretKey) -> Self {
        Self(BigInt::from_bytes(value.to_bytes()))
    }
}

impl FromStr for AccountSecretKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; 38];

        let size = bs58::decode(s)
            .with_check(Some(Self::BASE58_CHECK_VERSION))
            .into(&mut bytes)?;
        if size != 34 {
            return Err(bs58::decode::Error::BufferTooSmall.into());
        }

        Ok(Self::from_bytes(&bytes[2..34])?)
    }
}

impl fmt::Display for AccountSecretKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: implement to_bytes for Keypair, and remove this ugly workaround
        let hex = self.0.to_hex();
        let mut bytes = hex::decode(hex).expect("to_hex should return hex string");
        bytes.reverse();
        bytes.insert(0, 1);
        let s = bs58::encode(&bytes)
            .with_check_version(Self::BASE58_CHECK_VERSION)
            .into_string();
        f.write_str(&s)
    }
}

impl Serialize for AccountSecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for AccountSecretKey {
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
    use std::env;

    use super::*;

    #[test]
    fn test_account_secret_key_bs58check_decode() {
        let parsed: AccountSecretKey = "EKFWgzXsoMYcP1Hnj7dBhsefxNucZ6wyz676Qg5uMFNzytXAi2Ww"
            .parse()
            .unwrap();
        assert_eq!(
            parsed.0.get_address(),
            "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS"
        );
    }

    #[test]
    fn test_account_secret_key_display() {
        let parsed: AccountSecretKey = "EKFWgzXsoMYcP1Hnj7dBhsefxNucZ6wyz676Qg5uMFNzytXAi2Ww"
            .parse()
            .unwrap();
        assert_eq!(
            &parsed.to_string(),
            "EKFWgzXsoMYcP1Hnj7dBhsefxNucZ6wyz676Qg5uMFNzytXAi2Ww"
        );
    }

    #[test]
    fn test_encrypt_decrypt() {
        let password = "not-very-secure-pass";

        let new_key = AccountSecretKey::rand();
        let tmp_dir = env::temp_dir();
        let tmp_path = format!("{}/{}-key", tmp_dir.display(), new_key.public_key());

        // dump encrypted file
        new_key
            .to_encrypted_file(&tmp_path, password)
            .expect("Failed to encrypt secret key");

        // load and decrypt
        let decrypted = AccountSecretKey::from_encrypted_file(&tmp_path, password)
            .expect("Failed to decrypt secret key file");

        assert_eq!(
            new_key.public_key(),
            decrypted.public_key(),
            "Encrypted and decrypted public keys do not match"
        );
    }

    #[test]
    fn test_ocaml_key_decrypt() {
        let password = "not-very-secure-pass";
        let key_path = "../tests/files/accounts/test-key-1";
        let expected_public_key = "B62qmg7n4XqU3SFwx9KD9B7gxsKwxJP5GmxtBpHp1uxyN3grujii9a1";
        let decrypted = AccountSecretKey::from_encrypted_file(key_path, password)
            .expect("Failed to decrypt secret key file");

        assert_eq!(
            expected_public_key.to_string(),
            decrypted.public_key().to_string()
        )
    }
}

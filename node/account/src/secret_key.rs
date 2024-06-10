use std::{env, fmt, fs, path::PathBuf, str::FromStr};

use argon2::{password_hash::SaltString, Argon2, Params, PasswordHasher};
use base64::Engine;
use crypto_secretbox::aead::{Aead, OsRng};
use crypto_secretbox::{AeadCore, KeyInit, XSalsa20Poly1305};
use mina_p2p_messages::{bigint::BigInt, v2::SignatureLibPrivateKeyStableV1};
use mina_signer::{keypair::KeypairError, CompressedPubKey, Keypair};
use openmina_core::constants::GENESIS_PRODUCER_SK;
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
        (0..1000)
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

    pub fn rand() -> Self {
        Self::rand_with(rand::thread_rng())
    }

    pub fn rand_with(mut rng: impl Rng + CryptoRng) -> Self {
        Self(Keypair::rand(&mut rng))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeypairError> {
        Ok(Self(Keypair::from_bytes(bytes)?))
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

    pub fn from_encrypted_file(path: PathBuf) -> Result<Self, EncryptionError> {
        let key_file = fs::File::open(path)?;
        let encrypted: EncryptedSecretKey = serde_json::from_reader(key_file)?;
        encrypted.try_decrypt()
    }

    pub fn to_encrypted_file(&self, path: PathBuf) -> Result<(), EncryptionError> {
        if path.exists() {
            panic!("File {} already exists", path.display())
        }

        let f = fs::File::create(path)?;
        let encrypted = EncryptedSecretKey::encrypt(&self.to_bytes())?;

        serde_json::to_writer(f, &encrypted)?;
        Ok(())
    }
}

impl From<AccountSecretKey> for Keypair {
    fn from(value: AccountSecretKey) -> Self {
        value.0
    }
}

impl From<AccountSecretKey> for SignatureLibPrivateKeyStableV1 {
    fn from(value: AccountSecretKey) -> Self {
        Self(BigInt::new(value.to_bytes().into()))
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

#[derive(Serialize, Deserialize, Debug)]
struct Base58String(String);

impl Base58String {
    pub fn new(raw: &[u8], version: u8) -> Self {
        Base58String(bs58::encode(raw).with_check_version(version).into_string())
    }

    pub fn try_decode(&self, version: u8) -> Result<Vec<u8>, EncryptionError> {
        let decoded = bs58::decode(&self.0).with_check(Some(version)).into_vec()?;
        Ok(decoded[1..].to_vec())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error(transparent)]
    SecretBox(#[from] crypto_secretbox::aead::Error),
    #[error(transparent)]
    ArgonError(#[from] argon2::Error),
    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),
    #[error(transparent)]
    Base58DecodeError(#[from] bs58::decode::Error),
    #[error(transparent)]
    CipherKeyInvalidLength(#[from] crypto_secretbox::cipher::InvalidLength),
    #[error("Password hash missing after hash_password")]
    HashMissing,
    #[error(transparent)]
    Keypair(#[from] KeypairError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("MINA_PRIVKEY_PASS environment variable must be set!")]
    PasswordEnvVarMissing,
}

#[derive(Serialize, Deserialize, Debug)]
struct EncryptedSecretKey {
    box_primitive: String,
    pw_primitive: String,
    nonce: Base58String,
    pwsalt: Base58String,
    pwdiff: (u32, u32),
    ciphertext: Base58String,
}

impl EncryptedSecretKey {
    const ENCRYPTION_DATA_VERSION_BYTE: u8 = 2;
    const SECRET_KEY_PREFIX_BYTE: u8 = 1;

    // Based on the ocaml implementation
    const BOX_PRIMITIVE: &'static str = "xsalsa20poly1305";
    const PW_PRIMITIVE: &'static str = "argon2i";
    // Note: Only used for enryption, for decryption use the pwdiff from the file
    const PW_DIFF: (u32, u32) = (134217728, 6);

    fn setup_argon(pwdiff: (u32, u32)) -> Result<Argon2<'static>, EncryptionError> {
        let params = Params::new(pwdiff.0 / 1024, pwdiff.1, Params::DEFAULT_P_COST, None)?;

        Ok(Argon2::new(
            argon2::Algorithm::Argon2i,
            Default::default(),
            params,
        ))
    }

    pub fn try_decrypt(&self) -> Result<AccountSecretKey, EncryptionError> {
        // prepare inputs to cipher
        let password =
            env::var("MINA_PRIVKEY_PASS").map_err(|_| EncryptionError::PasswordEnvVarMissing)?;
        let password = password.as_bytes();
        let pwsalt = self.pwsalt.try_decode(Self::ENCRYPTION_DATA_VERSION_BYTE)?;
        let nonce = self.nonce.try_decode(Self::ENCRYPTION_DATA_VERSION_BYTE)?;
        let ciphertext = self
            .ciphertext
            .try_decode(Self::ENCRYPTION_DATA_VERSION_BYTE)?;

        // The argon crate's SaltString can only be built from base64 string, ocaml node encodes the salt in base58
        // So we decoded it from base58 first, then convert to base64 and lastly to SaltString
        let pwsalt_encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(pwsalt);
        let salt = SaltString::from_b64(&pwsalt_encoded)?;

        let argon2 = Self::setup_argon(self.pwdiff)?;
        let password_hash = argon2
            .hash_password(password, &salt)?
            .hash
            .ok_or(EncryptionError::HashMissing)?;
        let password_bytes = password_hash.as_bytes();

        // decrypt cipher
        let cipher = XSalsa20Poly1305::new_from_slice(password_bytes)?;
        let decrypted = cipher.decrypt(nonce.as_slice().into(), ciphertext.as_ref())?;

        // strip the prefix and create keypair
        Ok(AccountSecretKey::from_bytes(&decrypted[1..])?)
    }

    pub fn encrypt(key: &[u8]) -> Result<Self, EncryptionError> {
        let argon2 = Self::setup_argon(Self::PW_DIFF)?;

        // add the prefix byt to the key
        let mut key_prefixed = vec![Self::SECRET_KEY_PREFIX_BYTE];
        key_prefixed.extend(key);
        let password =
            env::var("MINA_PRIVKEY_PASS").map_err(|_| EncryptionError::PasswordEnvVarMissing)?;

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .hash
            .ok_or(EncryptionError::HashMissing)?;

        let nonce = XSalsa20Poly1305::generate_nonce(&mut OsRng);
        let cipher = XSalsa20Poly1305::new_from_slice(password_hash.as_bytes())?;

        let ciphertext = cipher.encrypt(&nonce, key_prefixed.as_slice())?;

        // Same reason as in decrypt, we ned to decode the SaltString from base64 then encode it to base58 bellow
        let mut salt_bytes = [0; 32];
        let salt_portion = salt.decode_b64(&mut salt_bytes)?;

        Ok(Self {
            box_primitive: Self::BOX_PRIMITIVE.to_string(),
            pw_primitive: Self::PW_PRIMITIVE.to_string(),
            nonce: Base58String::new(&nonce, Self::ENCRYPTION_DATA_VERSION_BYTE),
            pwsalt: Base58String::new(salt_portion, Self::ENCRYPTION_DATA_VERSION_BYTE),
            pwdiff: (argon2.params().m_cost() * 1024, argon2.params().t_cost()),
            ciphertext: Base58String::new(&ciphertext, Self::ENCRYPTION_DATA_VERSION_BYTE),
        })
    }
}

#[cfg(test)]
mod tests {
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
        env::set_var("MINA_PRIVKEY_PASS", "not-very-secure-pass");
        let new_key = AccountSecretKey::rand();
        let tmp_dir = env::temp_dir();
        let tmp_path = format!("{}/{}-key", tmp_dir.display(), new_key.public_key());

        // dump encrypted file
        new_key
            .to_encrypted_file(tmp_path.clone().into())
            .expect("Failed to encrypt secret key");

        // load and decrypt
        let decrypted = AccountSecretKey::from_encrypted_file(tmp_path.into())
            .expect("Failed to decrypt secret key file");

        assert_eq!(
            new_key.public_key(),
            decrypted.public_key(),
            "Encrypted and decrypted public keys do not match"
        );
    }

    #[test]
    fn test_ocaml_key_decrypt() {
        env::set_var("MINA_PRIVKEY_PASS", "not-very-secure-pass");
        let key_path = "../tests/files/accounts/test-key-1";
        let expected_public_key = "B62qmg7n4XqU3SFwx9KD9B7gxsKwxJP5GmxtBpHp1uxyN3grujii9a1";
        let decrypted = AccountSecretKey::from_encrypted_file(key_path.into())
            .expect("Failed to decrypt secret key file");

        assert_eq!(
            expected_public_key.to_string(),
            decrypted.public_key().to_string()
        )
    }
}

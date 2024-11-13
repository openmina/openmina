use std::fs;
use std::path::Path;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use base64::Engine;
use crypto_secretbox::aead::{Aead, OsRng};
use crypto_secretbox::{AeadCore, KeyInit, XSalsa20Poly1305};
use serde::{Deserialize, Serialize};

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
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("Other: {0}")]
    Other(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedSecretKeyFile {
    box_primitive: String,
    pw_primitive: String,
    nonce: Base58String,
    pwsalt: Base58String,
    pwdiff: (u32, u32),
    ciphertext: Base58String,
}

impl EncryptedSecretKeyFile {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, EncryptionError> {
        let file = fs::File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
}

fn setup_argon(pwdiff: (u32, u32)) -> Result<Argon2<'static>, EncryptionError> {
    let params = argon2::Params::new(
        pwdiff.0 / 1024,
        pwdiff.1,
        argon2::Params::DEFAULT_P_COST,
        None,
    )?;

    Ok(Argon2::new(
        argon2::Algorithm::Argon2i,
        Default::default(),
        params,
    ))
}

pub trait EncryptedSecretKey {
    const ENCRYPTION_DATA_VERSION_BYTE: u8 = 2;
    const SECRET_KEY_PREFIX_BYTE: u8 = 1;

    // Based on the ocaml implementation
    const BOX_PRIMITIVE: &'static str = "xsalsa20poly1305";
    const PW_PRIMITIVE: &'static str = "argon2i";
    // Note: Only used for enryption, for decryption use the pwdiff from the file
    const PW_DIFF: (u32, u32) = (134217728, 6);

    fn try_decrypt(
        encrypted: &EncryptedSecretKeyFile,
        password: &str,
    ) -> Result<Vec<u8>, EncryptionError> {
        // prepare inputs to cipher
        let password = password.as_bytes();
        let pwsalt = encrypted
            .pwsalt
            .try_decode(Self::ENCRYPTION_DATA_VERSION_BYTE)?;
        let nonce = encrypted
            .nonce
            .try_decode(Self::ENCRYPTION_DATA_VERSION_BYTE)?;
        let ciphertext = encrypted
            .ciphertext
            .try_decode(Self::ENCRYPTION_DATA_VERSION_BYTE)?;

        // The argon crate's SaltString can only be built from base64 string, ocaml node encodes the salt in base58
        // So we decoded it from base58 first, then convert to base64 and lastly to SaltString
        let pwsalt_encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(pwsalt);
        let salt = SaltString::from_b64(&pwsalt_encoded)?;

        let argon2 = setup_argon(encrypted.pwdiff)?;
        let password_hash = argon2
            .hash_password(password, &salt)?
            .hash
            .ok_or(EncryptionError::HashMissing)?;
        let password_bytes = password_hash.as_bytes();

        // decrypt cipher
        let cipher = XSalsa20Poly1305::new_from_slice(password_bytes)?;
        let decrypted = cipher.decrypt(nonce.as_slice().into(), ciphertext.as_ref())?;

        // strip the prefix and create keypair
        Ok(decrypted)
    }
    fn try_encrypt(key: &[u8], password: &str) -> Result<EncryptedSecretKeyFile, EncryptionError> {
        let argon2 = setup_argon(Self::PW_DIFF)?;

        // add the prefix byt to the key
        let mut key_prefixed = vec![Self::SECRET_KEY_PREFIX_BYTE];
        key_prefixed.extend(key);

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

        Ok(EncryptedSecretKeyFile {
            box_primitive: Self::BOX_PRIMITIVE.to_string(),
            pw_primitive: Self::PW_PRIMITIVE.to_string(),
            nonce: Base58String::new(&nonce, Self::ENCRYPTION_DATA_VERSION_BYTE),
            pwsalt: Base58String::new(salt_portion, Self::ENCRYPTION_DATA_VERSION_BYTE),
            pwdiff: (argon2.params().m_cost() * 1024, argon2.params().t_cost()),
            ciphertext: Base58String::new(&ciphertext, Self::ENCRYPTION_DATA_VERSION_BYTE),
        })
    }
}

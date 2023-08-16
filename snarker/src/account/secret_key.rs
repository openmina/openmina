use std::{str::FromStr, fmt};

use mina_signer::{keypair::KeypairError, Keypair};

use super::AccountPublicKey;

#[derive(Clone)]
pub struct AccountSecretKey(Keypair);

impl std::fmt::Debug for AccountSecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AccountSecretKey").field(&"***").finish()
    }
}

impl AccountSecretKey {
    const BASE58_CHECK_VERSION: u8 = 90;

    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();
        Self(Keypair::rand(&mut rng))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeypairError> {
        Ok(Self(Keypair::from_bytes(bytes)?))
    }

    pub fn public_key(&self) -> AccountPublicKey {
        self.0.public.clone().into()
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
        let mut bytes = hex::decode(&hex).expect("to_hex should return hex string");
        bytes.reverse();
        bytes.insert(0, 1);
        let s = bs58::encode(&bytes).with_check_version(Self::BASE58_CHECK_VERSION).into_string();
        f.write_str(&s)
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
        assert_eq!(&parsed.to_string(), "EKFWgzXsoMYcP1Hnj7dBhsefxNucZ6wyz676Qg5uMFNzytXAi2Ww");
    }
}

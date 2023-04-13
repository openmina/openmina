use std::str::FromStr;

use mina_signer::{keypair::KeypairError, Keypair};

use super::AccountPublicKey;

#[derive(Clone)]
pub struct AccountSecretKey(Keypair);

impl AccountSecretKey {
    const BASE58_CHECK_VERSION: u8 = 90;

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
        if dbg!(size) != 34 {
            return Err(bs58::decode::Error::BufferTooSmall.into());
        }

        Ok(Self::from_bytes(&bytes[2..34])?)
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
}

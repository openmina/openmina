use std::{fmt, str::FromStr};

use mina_p2p_messages::binprot::{
    self,
    macros::{BinProtRead, BinProtWrite},
};
use mina_p2p_messages::{
    b58::FromBase58CheckError,
    v2::{NonZeroCurvePoint, NonZeroCurvePointUncompressedStableV1},
};
use serde::{Deserialize, Serialize};

use mina_signer::{CompressedPubKey, PubKey};

#[derive(
    BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone,
)]
pub struct AccountPublicKey(NonZeroCurvePoint);

impl From<PubKey> for AccountPublicKey {
    fn from(value: PubKey) -> Self {
        value.into_compressed().into()
    }
}

impl From<CompressedPubKey> for AccountPublicKey {
    fn from(value: CompressedPubKey) -> Self {
        Self(
            NonZeroCurvePointUncompressedStableV1 {
                x: value.x.into(),
                is_odd: value.is_odd,
            }
            .into(),
        )
    }
}

impl TryFrom<AccountPublicKey> for CompressedPubKey {
    type Error = ();

    fn try_from(value: AccountPublicKey) -> Result<Self, Self::Error> {
        Ok(Self {
            is_odd: value.0.is_odd,
            x: value.0.into_inner().x.try_into().map_err(|_| ())?,
        })
    }
}

impl From<NonZeroCurvePoint> for AccountPublicKey {
    fn from(value: NonZeroCurvePoint) -> Self {
        Self(value)
    }
}

impl From<AccountPublicKey> for NonZeroCurvePoint {
    fn from(value: AccountPublicKey) -> Self {
        value.0
    }
}

impl FromStr for AccountPublicKey {
    type Err = FromBase58CheckError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl AsRef<NonZeroCurvePoint> for AccountPublicKey {
    fn as_ref(&self) -> &NonZeroCurvePoint {
        &self.0
    }
}

impl fmt::Display for AccountPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p2p_key = NonZeroCurvePoint::from(self.clone());
        write!(f, "{p2p_key}")
    }
}

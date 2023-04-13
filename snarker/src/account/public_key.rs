use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::{NonZeroCurvePoint, NonZeroCurvePointUncompressedStableV1};
use serde::{Deserialize, Serialize};

use mina_signer::{CompressedPubKey, PubKey};

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
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

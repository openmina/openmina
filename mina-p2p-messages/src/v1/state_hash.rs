use std::fmt;

use crate::bigint::BigInt;

use super::generated::{MinaStateProtocolStateValueStableV1VersionedV1PolyArg0, MinaStateProtocolStateValueStableV1VersionedV1PolyArg0V1, MinaStateProtocolStateValueStableV1VersionedV1PolyArg0V1Poly};

pub type StateHashStable = MinaStateProtocolStateValueStableV1VersionedV1PolyArg0;
pub type StateHashStableV1 = MinaStateProtocolStateValueStableV1VersionedV1PolyArg0V1;
pub type StateHashStableV1Poly = MinaStateProtocolStateValueStableV1VersionedV1PolyArg0V1Poly;

impl StateHashStableV1 {
    pub fn from_bigint(bigint: BigInt) -> Self {
        Self(StateHashStableV1Poly::from_bigint(bigint))
    }
}

impl StateHashStableV1Poly {
    pub fn from_bigint(bigint: BigInt) -> Self {
        Self(bigint)
    }
}

impl fmt::Display for StateHashStable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = vec![];
        binprot::BinProtWrite::binprot_write(self, &mut buf).or(Err(fmt::Error))?;

        bs58::encode(&buf)
            .with_check_version(0x10)
            .into_string()
            .fmt(f)
    }
}

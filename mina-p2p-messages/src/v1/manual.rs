use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::versioned::Versioned;

///! Manually implemented Mina types.

pub type NonzeroCurvePointV1Binable = Versioned<Versioned<NonzeroCurvePointV1, 1>, 1>;

pub type PublicKeyCompressedStableV1Binable = NonzeroCurvePointV1Binable;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NonzeroCurvePointV1 {
    x: crate::bigint::BigInt,
    is_odd: bool,
}

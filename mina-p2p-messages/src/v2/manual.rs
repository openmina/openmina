use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Serialize};

use crate::{b58::AsBase58Check, bigint::BigInt, versioned::Versioned};

use super::{
    ConsensusVrfOutputTruncatedStableV1, DataHashLibStateHashStableV1,
    MinaBaseAccountIdMakeStrDigestStableV1, MinaBaseEpochSeedStableV1, MinaBaseLedgerHash0StableV1,
    MinaBasePendingCoinbaseHashVersionedStableV1, MinaBaseStagedLedgerHashAuxHashStableV1,
    MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1, NonZeroCurvePointUncompressedStableV1,
    ParallelScanWeightStableV1, PicklesProofProofsVerified2ReprStableV2StatementFp,
    TransactionSnarkScanStateStableV2TreesABaseT1, TransactionSnarkScanStateStableV2TreesAMergeT1,
};

pub type TransactionSnarkScanStateStableV2TreesABase = (
    ParallelScanWeightStableV1,
    TransactionSnarkScanStateStableV2TreesABaseT1,
);

pub type TransactionSnarkScanStateStableV2TreesAMerge = (
    (ParallelScanWeightStableV1, ParallelScanWeightStableV1),
    TransactionSnarkScanStateStableV2TreesAMergeT1,
);

//
//  Location: [src/lib/parallel_scan/parallel_scan.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L247)
//
//  Gid: 947
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransactionSnarkScanStateStableV2TreesA {
    Leaf(Vec<TransactionSnarkScanStateStableV2TreesABase>),
    Node {
        depth: crate::number::Int32,
        value: Vec<TransactionSnarkScanStateStableV2TreesAMerge>,
        sub_tree: Box<TransactionSnarkScanStateStableV2TreesA>,
    },
}

#[derive(BinProtRead, BinProtWrite)]
enum _Tree {
    Leaf,
    Node,
}

impl BinProtRead for TransactionSnarkScanStateStableV2TreesA {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let mut depth: i32 = 0;
        let mut values: Vec<Vec<TransactionSnarkScanStateStableV2TreesAMerge>> = Vec::new();
        loop {
            match _Tree::binprot_read(r)? {
                _Tree::Leaf => {
                    let len = 1 << depth;
                    let mut data = Vec::with_capacity(len);
                    for _ in 0..len {
                        data.push(TransactionSnarkScanStateStableV2TreesABase::binprot_read(
                            r,
                        )?)
                    }
                    let mut tree = Self::Leaf(data);
                    while let Some(value) = values.pop() {
                        depth = depth - 1;
                        tree = Self::Node {
                            depth: depth.into(),
                            value,
                            sub_tree: Box::new(tree),
                        }
                    }
                    return Ok(tree);
                }
                _Tree::Node => {
                    let _depth = i32::binprot_read(r)?;
                    if _depth != depth {
                        return Err(binprot::Error::CustomError(
                            format!("Incorrect tree depth, expected `{depth}`, got `{_depth}`")
                                .into(),
                        ));
                    }
                    let len = 1 << depth;
                    let mut value = Vec::with_capacity(len);
                    for _ in 0..len {
                        value.push(TransactionSnarkScanStateStableV2TreesAMerge::binprot_read(
                            r,
                        )?)
                    }
                    values.push(value);
                    depth += 1;
                }
            }
        }
    }
}

impl BinProtWrite for TransactionSnarkScanStateStableV2TreesA {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let mut curr = self;
        let mut curr_depth = 0;
        loop {
            match curr {
                Self::Leaf(leaf) => {
                    let len = 1 << curr_depth;
                    if leaf.len() != len {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Invalid leaf data lenght, expecting {len}, got {}",
                                leaf.len()
                            ),
                        ));
                    }
                    _Tree::Leaf.binprot_write(w)?;
                    leaf.iter().try_for_each(|b| b.binprot_write(w))?;
                    return Ok(());
                }
                Self::Node {
                    depth,
                    value,
                    sub_tree,
                } => {
                    if &depth.0 != &curr_depth {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Invalid depth, expecting {curr_depth}, got {depth}",
                                depth = depth.0
                            ),
                        ));
                    }
                    if value.len() != (1 << curr_depth) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Invalid node data lenght, expecting {}, got {}",
                                1 << curr_depth,
                                value.len()
                            ),
                        ));
                    }
                    _Tree::Node.binprot_write(w)?;
                    depth.binprot_write(w)?;
                    value.iter().try_for_each(|b| b.binprot_write(w))?;
                    curr = sub_tree;
                    curr_depth += 1;
                }
            }
        }
    }
}

macro_rules! hash {
    ($name:ident, $ty:ty, $version:expr, $version_byte:ident) => {
        impl From<Versioned<$ty, $version>> for $ty {
            fn from(source: Versioned<$ty, $version>) -> Self {
                source.into_inner()
            }
        }

        pub type $name =
            AsBase58Check<$ty, Versioned<$ty, $version>, { $crate::b58version::$version_byte }>;
    };
    ($name:ident, $ty:ty, $version_byte:ident) => {
        impl From<Versioned<$ty, 1>> for $ty {
            fn from(source: Versioned<$ty, 1>) -> Self {
                source.into_inner()
            }
        }

        pub type $name =
            AsBase58Check<$ty, Versioned<$ty, 1>, { $crate::b58version::$version_byte }>;
    };
}

hash!(LedgerHash, MinaBaseLedgerHash0StableV1, LEDGER_HASH);
hash!(
    StagedLedgerHashAuxHash,
    MinaBaseStagedLedgerHashAuxHashStableV1,
    STAGED_LEDGER_HASH_AUX_HASH
);
hash!(EpochSeed, MinaBaseEpochSeedStableV1, EPOCH_SEED);
hash!(
    StagedLedgerHashPendingCoinbaseAux,
    MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1,
    STAGED_LEDGER_HASH_PENDING_COINBASE_AUX
);
hash!(StateHash, DataHashLibStateHashStableV1, STATE_HASH);
hash!(
    PendingCoinbaseHash,
    MinaBasePendingCoinbaseHashVersionedStableV1,
    RECEIPT_CHAIN_HASH
);
hash!(
    TokenIdKeyHash,
    MinaBaseAccountIdMakeStrDigestStableV1,
    TOKEN_ID_KEY
);
hash!(
    VrfTruncatedOutput,
    ConsensusVrfOutputTruncatedStableV1,
    VRF_TRUNCATED_OUTPUT
);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NonZeroCurvePointWithVersions {
    x: Versioned<crate::bigint::BigInt, 1>,
    is_odd: bool,
}

impl From<NonZeroCurvePointUncompressedStableV1> for Versioned<NonZeroCurvePointWithVersions, 1> {
    fn from(source: NonZeroCurvePointUncompressedStableV1) -> Self {
        NonZeroCurvePointWithVersions {
            x: source.x.into(),
            is_odd: source.is_odd,
        }
        .into()
    }
}

impl From<Versioned<NonZeroCurvePointWithVersions, 1>> for NonZeroCurvePointUncompressedStableV1 {
    fn from(source: Versioned<NonZeroCurvePointWithVersions, 1>) -> Self {
        let source = source.into_inner();
        Self {
            x: source.x.into_inner(),
            is_odd: source.is_odd,
        }
    }
}

pub type NonZeroCurvePoint = AsBase58Check<
    NonZeroCurvePointUncompressedStableV1,
    Versioned<NonZeroCurvePointWithVersions, 1>,
    { crate::b58version::NON_ZERO_CURVE_POINT_COMPRESSED },
>;

#[cfg(test)]
mod tests {
    use super::NonZeroCurvePoint;

    #[test]
    fn non_zero_curve_point() {
        let b58 = r#""B62qkUHaJUHERZuCHQhXCQ8xsGBqyYSgjQsKnKN5HhSJecakuJ4pYyk""#;

        let v = serde_json::from_str::<NonZeroCurvePoint>(&b58)
            .unwrap()
            .into_inner();
        assert_eq!(v.is_odd, false);
        assert_eq!(
            &hex::encode(&v.x),
            "3c2b5b48c22dc8b8c9d2c9d76a2ceaaf02beabb364301726c3f8e989653af513"
        );
    }
}

const SHIFTED_VALUE: &str = "Shifted_value";

impl Serialize for PicklesProofProofsVerified2ReprStableV2StatementFp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let mut serializer = serializer.serialize_tuple(2)?;
            match self {
                PicklesProofProofsVerified2ReprStableV2StatementFp::ShiftedValue(v) => {
                    serializer.serialize_element(SHIFTED_VALUE)?;
                    serializer.serialize_element(v)?;
                }
            }
            serializer.end()
        } else {
            match self {
                PicklesProofProofsVerified2ReprStableV2StatementFp::ShiftedValue(v) => serializer
                    .serialize_newtype_variant(
                        "PicklesProofProofsVerified2ReprStableV2StatementFp",
                        0,
                        "ShiftedValue",
                        v,
                    ),
            }
        }
    }
}

impl<'de> Deserialize<'de> for PicklesProofProofsVerified2ReprStableV2StatementFp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = PicklesProofProofsVerified2ReprStableV2StatementFp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("tuple of tag and value")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                match seq.next_element::<String>()? {
                    Some(v) if &v == SHIFTED_VALUE => {}
                    Some(v) => {
                        return Err(serde::de::Error::custom(format!(
                            "expecting `{SHIFTED_VALUE}`, got `{v}`"
                        )))
                    }
                    None => return Err(serde::de::Error::custom("expecting a tag")),
                }
                match seq.next_element::<BigInt>()? {
                    Some(v) => {
                        Ok(PicklesProofProofsVerified2ReprStableV2StatementFp::ShiftedValue(v))
                    }
                    None => return Err(serde::de::Error::custom("expecting a value")),
                }
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_tuple(2, V)
        } else {
            todo!()
        }
    }
}

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Serialize};

use crate::{b58::AsBase58Check, bigint::BigInt, string::ByteString, versioned::Versioned};

use super::{
    ConsensusVrfOutputTruncatedStableV1, DataHashLibStateHashStableV1,
    MinaBaseAccountIdDigestStableV1, MinaBaseEpochSeedStableV1, MinaBaseLedgerHash0StableV1,
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
    ($name:ident, versioned($ty:ty, $version:expr), $version_byte:ident) => {
        impl From<Versioned<$ty, $version>> for $ty {
            fn from(source: Versioned<$ty, $version>) -> Self {
                source.into_inner()
            }
        }

        pub type $name =
            AsBase58Check<$ty, Versioned<$ty, $version>, { $crate::b58version::$version_byte }>;
    };
    ($name:ident, versioned $ty:ty, $version_byte:ident) => {
        hash!($name, versioned($ty, 1), $version_byte);
    };
    ($name:ident, $ty:ty, $version_byte:ident) => {
        pub type $name = AsBase58Check<$ty, $ty, { $crate::b58version::$version_byte }>;
    };
}

hash!(LedgerHash, versioned MinaBaseLedgerHash0StableV1, LEDGER_HASH);
hash!(
    StagedLedgerHashAuxHash,
    MinaBaseStagedLedgerHashAuxHashStableV1,
    STAGED_LEDGER_HASH_AUX_HASH
);
hash!(EpochSeed, versioned MinaBaseEpochSeedStableV1, EPOCH_SEED);
hash!(
    StagedLedgerHashPendingCoinbaseAux,
    MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1,
    STAGED_LEDGER_HASH_PENDING_COINBASE_AUX
);
hash!(StateHash, versioned DataHashLibStateHashStableV1, STATE_HASH);
hash!(
    PendingCoinbaseHash,
    versioned MinaBasePendingCoinbaseHashVersionedStableV1,
    RECEIPT_CHAIN_HASH
);
hash!(
    TokenIdKeyHash,
    MinaBaseAccountIdDigestStableV1,
    TOKEN_ID_KEY
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
    use binprot::BinProtWrite;
    use serde::{de::DeserializeOwned, Serialize};

    use super::*;

    fn base58check_test<T: Serialize + DeserializeOwned + BinProtWrite>(b58: &str, hex: &str) {
        let bin: T = serde_json::from_value(serde_json::json!(b58)).unwrap();
        let json = serde_json::to_value(&bin).unwrap();

        let mut binprot = Vec::new();
        bin.binprot_write(&mut binprot).unwrap();

        // println!("{b58} => {}", hex::encode(&binprot));
        // println!("{hex} => {}", json.as_str().unwrap());

        assert_eq!(hex::encode(&binprot), hex);
        assert_eq!(json.as_str().unwrap(), b58);
    }

    macro_rules! b58t {
        ($name:ident, $ty:ty, $b58:expr, $hex:expr) => {
            #[test]
            fn $name() {
                base58check_test::<$ty>($b58, $hex);
            }
        };
    }

    b58t!(
        ledger_hash,
        LedgerHash,
        "jwrPvAMUNo3EKT2puUk5Fxz6B7apRAoKNTGpAA49t3TRSfzvdrL",
        "636f5b2d67278e17bc4343c7c23fb4991f8cf0bbbfd8558615b124d5d6254801"
    );

    b58t!(
        staged_ledger_hash_aux_hash,
        StagedLedgerHashAuxHash,
        "36DghpXjs8tswr8NCeWkLmsSW52gH98qbuAXcfFsr6ykCdvjKwoF",
        "20d9b3ceaed3f1dc7a13fcd88585fefadb86475d83e89828a7099e2ee5506d173c"
    );

    b58t!(
        epoch_seed,
        EpochSeed,
        "2vajKi2Cxx58mByzxbJA3G6gYh1j2BoizW4zzoLcZa3kYECjhaXV",
        "4d8802db5beb98f13e10475ddc9e718f6890613276331c062f5d71b915d6941d"
    );

    b58t!(
        staged_ledger_hash_pending_coinbase_aux,
        StagedLedgerHashPendingCoinbaseAux,
        "3EoSZGcx4LkUtnsEwaZfUt5VcXjJbiDKybGTbTjFsZT5MBKzH3eU",
        "20ea3c01998507afba1adb2a8d14831fd5af3ff687078bfc0c68916abc98e76384"
    );

    b58t!(
        state_hash,
        StateHash,
        "3NL7AkynW6hbDrhHTAht1GLG563Fo9fdcEQk1zEyy5XedC6aZTeB",
        "8d67aadd018581a812623915b13d5c3a6da7dfe8a195172d9bbd206810bc2329"
    );

    b58t!(
        pending_coinbase_hash,
        PendingCoinbaseHash,
        "2n2EEn3yH1oRU8tCXTjw7dJKHQVcFTkfeDCTpBzum3sZcssPeaVM",
        "e23a19254e600402e4474371450d498c75a9b3e28c34160d489af61c255f722c"
    );

    b58t!(
        token_id_key,
        TokenIdKeyHash,
        "wSHV2S4qX9jFsLjQo8r1BsMLH2ZRKsZx6EJd1sbozGPieEC4Jf",
        "0100000000000000000000000000000000000000000000000000000000000000"
    );

    b58t!(
        vrf_truncated_output,
        ConsensusVrfOutputTruncatedStableV1,
        "a5iclEJ9uqh_etVYuaL4MRWJ--1DFGsqp8CrDzNOGwM=",
        "206b989c94427dbaa87f7ad558b9a2f8311589fbed43146b2aa7c0ab0f334e1b03"
    );

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

impl Serialize for ConsensusVrfOutputTruncatedStableV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let base64 = base64::encode_config(&self.0, base64::URL_SAFE);
            base64.serialize(serializer)
        } else {
            todo!()
        }
    }
}

impl<'de> Deserialize<'de> for ConsensusVrfOutputTruncatedStableV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let base64 = String::deserialize(deserializer)?;
            base64::decode_config(&base64, base64::URL_SAFE)
                .map(ByteString::from)
                .map_err(|e| serde::de::Error::custom(format!("Error deserializing vrf: {e}")))
        } else {
            todo!()
        }
        .map(Self)
    }
}

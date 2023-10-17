use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::Deref;
use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Serialize, Serializer};

use crate::{
    b58::Base58CheckOfBinProt, b58::Base58CheckOfBytes, bigint::BigInt, string::ByteString,
    versioned::Versioned,
};

use super::{
    ConsensusProofOfStakeDataConsensusStateValueStableV2, ConsensusVrfOutputTruncatedStableV1,
    CurrencyFeeStableV1, DataHashLibStateHashStableV1, MinaBaseAccountIdDigestStableV1,
    MinaBaseEpochSeedStableV1, MinaBaseLedgerHash0StableV1,
    MinaBasePendingCoinbaseCoinbaseStackStableV1, MinaBasePendingCoinbaseHashVersionedStableV1,
    MinaBasePendingCoinbaseStackHashStableV1, MinaBaseSignatureStableV1,
    MinaBaseStateBodyHashStableV1, NonZeroCurvePointUncompressedStableV1,
    ParallelScanWeightStableV1, PicklesProofProofsVerified2ReprStableV2,
    PicklesProofProofsVerified2ReprStableV2StatementFp, PicklesProofProofsVerifiedMaxStableV2,
    ProtocolVersionStableV1, SgnStableV1, TransactionSnarkScanStateStableV2ScanStateTreesABaseT1,
    TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1,
};

pub type TransactionSnarkScanStateStableV2TreesABase = (
    ParallelScanWeightStableV1,
    TransactionSnarkScanStateStableV2ScanStateTreesABaseT1,
);

pub type TransactionSnarkScanStateStableV2TreesAMerge = (
    (ParallelScanWeightStableV1, ParallelScanWeightStableV1),
    TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1,
);

/// **OCaml name**: `Mina_base__Signed_command_memo.Make_str.Stable.V1`
///
/// Gid: `695`
/// Location: [src/lib/mina_base/signed_command_memo.ml:21:6](https://github.com/MinaProtocol/mina/blob//bfd1009/src/lib/mina_base/signed_command_memo.ml#L21)
///
///
/// Gid: `170`
/// Location: [src/std_internal.ml:140:2](https://github.com/MinaProtocol/mina/blob//bfd1009/src/std_internal.ml#L140)
///
///
/// Gid: `83`
/// Location: [src/string.ml:44:6](https://github.com/MinaProtocol/mina/blob//bfd1009/src/string.ml#L44)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite, Deref)]
pub struct MinaBaseSignedCommandMemoStableV1(pub crate::string::CharString);

//
//  Location: [src/lib/parallel_scan/parallel_scan.ml:247:6](https://github.com/openmina/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L247)
//
//  Gid: 947
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransactionSnarkScanStateStableV2ScanStateTreesA {
    Leaf(Vec<TransactionSnarkScanStateStableV2TreesABase>),
    Node {
        depth: crate::number::Int32,
        value: Vec<TransactionSnarkScanStateStableV2TreesAMerge>,
        sub_tree: Box<TransactionSnarkScanStateStableV2ScanStateTreesA>,
    },
}

#[derive(BinProtRead, BinProtWrite)]
enum _Tree {
    Leaf,
    Node,
}

impl BinProtRead for TransactionSnarkScanStateStableV2ScanStateTreesA {
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

impl BinProtWrite for TransactionSnarkScanStateStableV2ScanStateTreesA {
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

macro_rules! base58check_of_binprot {
    ($name:ident, versioned($ty:ty, $version:expr), $version_byte:ident) => {
        impl From<Versioned<$ty, $version>> for $ty {
            fn from(source: Versioned<$ty, $version>) -> Self {
                source.into_inner()
            }
        }

        pub type $name = Base58CheckOfBinProt<
            $ty,
            Versioned<$ty, $version>,
            { $crate::b58version::$version_byte },
        >;
    };
    ($name:ident, versioned $ty:ty, $version_byte:ident) => {
        base58check_of_binprot!($name, versioned($ty, 1), $version_byte);
    };
    ($name:ident, $ty:ty, $version_byte:ident) => {
        pub type $name = Base58CheckOfBinProt<$ty, $ty, { $crate::b58version::$version_byte }>;
    };
}

macro_rules! base58check_of_bytes {
    ($name:ident, $ty:ty, $version_byte:ident) => {
        pub type $name = Base58CheckOfBytes<$ty, { $crate::b58version::$version_byte }>;
    };
}

base58check_of_binprot!(LedgerHash, versioned MinaBaseLedgerHash0StableV1, LEDGER_HASH);
base58check_of_bytes!(
    StagedLedgerHashAuxHash,
    crate::string::ByteString,
    STAGED_LEDGER_HASH_AUX_HASH
);
base58check_of_binprot!(EpochSeed, versioned MinaBaseEpochSeedStableV1, EPOCH_SEED);
base58check_of_bytes!(
    StagedLedgerHashPendingCoinbaseAux,
    crate::string::ByteString,
    STAGED_LEDGER_HASH_PENDING_COINBASE_AUX
);
base58check_of_binprot!(StateHash, versioned DataHashLibStateHashStableV1, STATE_HASH);
base58check_of_binprot!(StateBodyHash, versioned MinaBaseStateBodyHashStableV1, STATE_BODY_HASH);
base58check_of_binprot!(
    PendingCoinbaseHash,
    versioned MinaBasePendingCoinbaseHashVersionedStableV1,
    RECEIPT_CHAIN_HASH
);
base58check_of_binprot!(
    TokenIdKeyHash,
    MinaBaseAccountIdDigestStableV1,
    TOKEN_ID_KEY
);
base58check_of_binprot!(
    CoinbaseStackData,
    versioned MinaBasePendingCoinbaseCoinbaseStackStableV1,
    COINBASE_STACK_DATA
);
base58check_of_binprot!(
    CoinbaseStackHash,
    versioned MinaBasePendingCoinbaseStackHashStableV1,
    COINBASE_STACK_HASH
);
base58check_of_binprot!(
    Signature,
    versioned MinaBaseSignatureStableV1,
    SIGNATURE
);

impl AsRef<[u8]> for LedgerHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, BinProtRead, BinProtWrite,
)]
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

pub type NonZeroCurvePoint = Base58CheckOfBinProt<
    NonZeroCurvePointUncompressedStableV1,
    Versioned<NonZeroCurvePointWithVersions, 1>,
    { crate::b58version::NON_ZERO_CURVE_POINT_COMPRESSED },
>;

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use binprot::{BinProtRead, BinProtWrite};
    use serde::{de::DeserializeOwned, Serialize};

    use super::*;

    fn base58check_test<T: Serialize + DeserializeOwned + BinProtRead + BinProtWrite + Debug>(
        b58: &str,
        hex: &str,
    ) {
        let bin: T = serde_json::from_value(serde_json::json!(b58)).unwrap();
        let json = serde_json::to_value(&bin).unwrap();

        let mut binprot = Vec::new();
        bin.binprot_write(&mut binprot).unwrap();

        println!("{b58} => {}", hex::encode(&binprot));

        let binprot1 = hex::decode(hex).unwrap();
        let mut b = binprot1.as_slice();
        let from_hex = T::binprot_read(&mut b).unwrap();
        println!("{hex} => {}", serde_json::to_string(&from_hex).unwrap());

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
        "UbhWTJLi4JM5bizFQVPvfMbjh4tCiTUTrNhedn8WdMPR1KHKJh",
        "203294e118730ad8b7c0f2ab6d74d244eec02cfef221790bb1274fdb3b97854e50"
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
        "XgkNHpgSvmF7CyRBGUzcwvCD9daBRhZUDLg3xTvohmTX1mRqHR",
        "20c922885bfeda2c2568e32fcc595fe0ad2292dcf756be637545bc553f7f7028e8"
    );

    b58t!(
        state_hash,
        StateHash,
        "3NL7AkynW6hbDrhHTAht1GLG563Fo9fdcEQk1zEyy5XedC6aZTeB",
        "8d67aadd018581a812623915b13d5c3a6da7dfe8a195172d9bbd206810bc2329"
    );

    b58t!(
        state_body_hash,
        StateBodyHash,
        "3WtsPNWF7Ua5zbvHEARuEL32KEfMM7pPYNXWVGtTStdYJRYA2rta",
        "1b11c26e5541d2f719a50f2e5bdcd23e7995883036f3d2e5675dfd3015ec6202"
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

    b58t!(
        coinbase_stack_data,
        CoinbaseStackData,
        "4QNrZFBTDQCPfEZqBZsaPYx8qdaNFv1nebUyCUsQW9QUJqyuD3un",
        "35b9d51e5d7c741456f86720731241a8280273cfc6c21668fd7bc6c587d0cc1d"
    );

    b58t!(
        coinbase_stack_hash,
        CoinbaseStackHash,
        "4Yx5U3t3EYQycZ91yj4478bHkLwGkhDHnPbCY9TxgUk69SQityej",
        "0000000000000000000000000000000000000000000000000000000000000000"
    );

    b58t!(
        signature,
        Signature,
        "7mXS9Y91bWtTYNKuDbxTuG18wUiZLHUySy9Ms8bPyAT9KNnME1q2nctwnvowJi2Y79dnsL18iVSCuaQF1ufUKwUZZKAXHqnF",
        "d290f924705fb714e91fedb9bed77e85bce8f5d932c3f4d692b20e4c3e5f9a3343c2baffce9ab0c2391e2f3de8ac891633338d827e6fd4f269331c248029b106"
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

impl<'de> Deserialize<'de> for ProtocolVersionStableV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;

            let err = || serde::de::Error::custom(format!("incorrect protocol version '{}'", s));

            let parse_number =
                |s: Option<&str>| s.and_then(|s| s.parse::<i64>().ok()).ok_or_else(err);

            let mut versions = s.split('.');
            let major = parse_number(versions.next())?.into();
            let minor = parse_number(versions.next())?.into();
            let patch = parse_number(versions.next())?.into();

            if versions.next().is_some() {
                return Err(err()); // We expect the format "major.minor.patch"
            }

            Ok(Self {
                major,
                minor,
                patch,
            })
        } else {
            todo!()
        }
    }
}

impl Serialize for ProtocolVersionStableV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let s = format!("{}.{}.{}", *self.major, *self.minor, *self.patch);
            s.serialize(serializer)
        } else {
            todo!()
        }
    }
}

pub type MerkleTreePath = Vec<MerkleTreeNode>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[polymorphic_variant]
pub enum MerkleTreeNode {
    Left(BigInt),
    Right(BigInt),
}

impl ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    pub fn global_slot(&self) -> u32 {
        match &self.curr_global_slot.slot_number {
            super::MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(s) => s.as_u32(),
        }
    }
}

impl AsRef<str> for SgnStableV1 {
    fn as_ref(&self) -> &str {
        match self {
            SgnStableV1::Pos => "Pos",
            SgnStableV1::Neg => "Neg",
        }
    }
}

impl Serialize for SgnStableV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let mut tuple = serializer.serialize_tuple(1)?;
            tuple.serialize_element(self.as_ref())?;
            tuple.end()
        } else {
            match *self {
                SgnStableV1::Pos => {
                    Serializer::serialize_unit_variant(serializer, "SgnStableV1", 0u32, "Pos")
                }
                SgnStableV1::Neg => {
                    Serializer::serialize_unit_variant(serializer, "SgnStableV1", 1u32, "Neg")
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for SgnStableV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = String;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("`Pos` or `Neg`")?;
                    panic!("foo")
                }

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(v)
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let Some(elt) = seq.next_element()? else { return Err(serde::de::Error::custom("No tag")); };
                    Ok(elt)
                }
            }
            let v = deserializer.deserialize_tuple(1, V)?;
            match v.as_str() {
                "Pos" => Ok(SgnStableV1::Pos),
                "Neg" => Ok(SgnStableV1::Neg),
                _ => Err(serde::de::Error::custom(format!("Invalid tag {v}"))),
            }
        } else {
            #[derive(Deserialize)]
            enum _SgnStableV1 {
                Pos,
                Neg,
            }

            let s: _SgnStableV1 = Deserialize::deserialize(deserializer)?;
            match s {
                _SgnStableV1::Pos => Ok(SgnStableV1::Pos),
                _SgnStableV1::Neg => Ok(SgnStableV1::Neg),
            }
        }
    }
}

#[derive(Deserialize)]
struct Foo(SgnStableV1);

#[cfg(test)]
mod tests_sgn {
    use crate::v2::SgnStableV1;

    #[test]
    fn test_json() {
        assert_eq!(
            serde_json::to_value(&SgnStableV1::Pos).unwrap(),
            serde_json::json!(["Pos"])
        );
        assert_eq!(
            serde_json::to_value(&SgnStableV1::Neg).unwrap(),
            serde_json::json!(["Neg"])
        );

        assert_eq!(
            serde_json::from_value::<SgnStableV1>(serde_json::json!(["Pos"])).unwrap(),
            SgnStableV1::Pos
        );
        assert_eq!(
            serde_json::from_value::<SgnStableV1>(serde_json::json!(["Neg"])).unwrap(),
            SgnStableV1::Neg
        );
    }
}

/// Derived name: `Mina_base__Fee_excess.Stable.V1.fee`
///
/// Gid: `602`
/// Location: [src/lib/currency/signed_poly.ml:6:4](https://github.com/Minaprotocol/mina/blob/b1facec/src/lib/currency/signed_poly.ml#L6)
/// Args: CurrencyFeeStableV1 , SgnStableV1
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct SignedAmount {
    pub magnitude: CurrencyFeeStableV1,
    pub sgn: SgnStableV1,
}

/// **OCaml name**: `Mina_base__Fee_excess.Stable.V1`
///
/// Gid: `657`
/// Location: [src/lib/mina_base/fee_excess.ml:124:4](https://github.com/Minaprotocol/mina/blob/b1facec/src/lib/mina_base/fee_excess.ml#L124)
///
///
/// Gid: `656`
/// Location: [src/lib/mina_base/fee_excess.ml:54:6](https://github.com/Minaprotocol/mina/blob/b1facec/src/lib/mina_base/fee_excess.ml#L54)
/// Args: TokenIdKeyHash , MinaBaseFeeExcessStableV1Fee
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct MinaBaseFeeExcessStableV1(pub TokenFeeExcess, pub TokenFeeExcess);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct TokenFeeExcess {
    pub token: TokenIdKeyHash,
    pub amount: SignedAmount,
}

impl Default for NonZeroCurvePointUncompressedStableV1 {
    fn default() -> Self {
        Self {
            x: Default::default(),
            is_odd: false,
        }
    }
}

impl super::MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    pub fn as_u32(&self) -> u32 {
        let Self::SinceGenesis(slot) = self;
        slot.as_u32()
    }
}

impl super::MinaNumbersGlobalSlotSinceHardForkMStableV1 {
    pub fn as_u32(&self) -> u32 {
        let Self::SinceHardFork(slot) = self;
        slot.as_u32()
    }
}

impl super::MinaNumbersGlobalSlotSpanStableV1 {
    pub fn as_u32(&self) -> u32 {
        let Self::GlobalSlotSpan(slot) = self;
        slot.as_u32()
    }
}

impl From<u32> for super::UnsignedExtendedUInt32StableV1 {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<&PicklesProofProofsVerifiedMaxStableV2> for PicklesProofProofsVerified2ReprStableV2 {
    fn from(value: &PicklesProofProofsVerifiedMaxStableV2) -> Self {
        let PicklesProofProofsVerifiedMaxStableV2 {
            statement,
            prev_evals,
            proof,
        } = value;

        Self {
            statement: statement.clone(),
            prev_evals: prev_evals.clone(),
            proof: proof.clone(),
        }
    }
}

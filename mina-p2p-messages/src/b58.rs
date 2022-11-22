//! Base58check encoding/decoding.

use std::marker::PhantomData;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::From;
use serde::{Deserialize, Serialize};

/// Before encoding, data is prepended with the version byte.
pub fn encode(b: &[u8], v: u8) -> String {
    bs58::encode(&b).with_check_version(v).into_string()
}

/// When decoded, resulting data is checked against the version byte as its
/// prefix.
///
/// Note that the prefix is still the part of the result.
pub fn decode(s: &str, v: u8) -> Result<Vec<u8>, bs58::decode::Error> {
    bs58::decode(s).with_check(Some(v)).into_vec()
}

/// The type that can be converted to base58check representation.
pub trait ToBase58Check: Sized {
    type Error;

    /// Produces base58check representation of this value.
    fn to_base58check(&self) -> Result<String, Self::Error>;
}

/// The type that can be constructed from base58check representation.
pub trait FromBase58Check: Sized {
    type Error;

    /// Constructs this instance from base58check representation.
    fn from_base58check<T: AsRef<str>>(b58: T) -> Result<Self, Self::Error>;
}

pub trait Base58CheckVersion {
    const VERSION_BYTE: u8;
}

#[derive(Debug, thiserror::Error)]
pub enum ToBase58CheckError {
    #[error("Error writing the value to binprot: {0}")]
    Binprot(#[from] std::io::Error),
}

impl<T> ToBase58Check for T
where
    T: BinProtWrite + Base58CheckVersion,
{
    type Error = ToBase58CheckError;

    fn to_base58check(&self) -> Result<String, Self::Error> {
        let mut binprot = Vec::new();
        self.binprot_write(&mut binprot)?;
        Ok(encode(&binprot, Self::VERSION_BYTE))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FromBase58CheckError {
    #[error("Error reading the value from binprot: {0}")]
    Binprot(#[from] binprot::Error),
    #[error("Error converting base58check to the value: {0}")]
    Base58(#[from] bs58::decode::Error),
}

impl<T> FromBase58Check for T
where
    T: BinProtRead + Base58CheckVersion,
{
    type Error = FromBase58CheckError;

    fn from_base58check<S: AsRef<str>>(b58: S) -> Result<Self, Self::Error> {
        let binprot = decode(b58.as_ref(), Self::VERSION_BYTE)?;
        let binable = T::binprot_read(&mut &binprot[..])?;
        Ok(binable)
    }
}

/// Wrapper that uses base58check of binprot serialization for the wrapped type
/// for human readable serializer.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Base58CheckOfBinProt<T, U, const V: u8>(T, PhantomData<U>);

impl<T, U, const V: u8> Clone for Base58CheckOfBinProt<T, U, V>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), Default::default())
    }
}

impl<T, U, const V: u8> From<T> for Base58CheckOfBinProt<T, U, V> {
    fn from(source: T) -> Self {
        Base58CheckOfBinProt(source, Default::default())
    }
}

impl<T, U, const V: u8> Base58CheckOfBinProt<T, U, V> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, U, const V: u8> BinProtRead for Base58CheckOfBinProt<T, U, V>
where
    T: BinProtRead,
{
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        T::binprot_read(r).map(|v| Self(v, Default::default()))
    }
}

impl<T, U, const V: u8> BinProtWrite for Base58CheckOfBinProt<T, U, V>
where
    T: BinProtWrite,
{
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.0.binprot_write(w)
    }
}

impl<T, U, const V: u8> Serialize for Base58CheckOfBinProt<T, U, V>
where
    T: Clone + Serialize,
    U: From<T> + BinProtWrite + std::fmt::Debug,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let mut binprot = Vec::new();
            let from = U::from(self.0.clone());
            from.binprot_write(&mut binprot).map_err(|e| {
                serde::ser::Error::custom(format!("Failed to convert to base58check: {e}"))
            })?;
            let encoded = encode(&binprot, V);
            serializer.serialize_str(&encoded)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de, T, U, const V: u8> serde::Deserialize<'de> for Base58CheckOfBinProt<T, U, V>
where
    T: From<U> + Deserialize<'de>,
    U: BinProtRead,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let b58: String = Deserialize::deserialize(deserializer)?;
            let binprot = decode(&b58, V).map_err(|e| {
                serde::de::Error::custom(format!("Failed to construct from base58check: {e}"))
            })?;
            let binable = U::binprot_read(&mut &binprot[1..]).map_err(|e| {
                serde::de::Error::custom(format!("Failed to construct from base58check: {e}"))
            })?;
            Ok(binable.into())
        } else {
            T::deserialize(deserializer)
        }
        .map(|v| Self(v, Default::default()))
    }
}

/// Wrapper that uses base58check of byte representation for the wrapped type
/// for human readable serializer.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, From, BinProtRead, BinProtWrite)]
pub struct Base58CheckOfBytes<T, const V: u8>(T);

impl<T, const V: u8> Base58CheckOfBytes<T, V> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, const V: u8> Serialize for Base58CheckOfBytes<T, V>
where
    T: Serialize + AsRef<[u8]> + std::fmt::Debug,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let encoded = encode(self.0.as_ref(), V);
            serializer.serialize_str(&encoded)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de, T, const V: u8> serde::Deserialize<'de> for Base58CheckOfBytes<T, V>
where
    T: for<'a> From<&'a [u8]> + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let b58: String = Deserialize::deserialize(deserializer)?;
            let bytes = decode(&b58, V).map_err(|e| {
                serde::de::Error::custom(format!("Failed to construct from base58check: {e}"))
            })?;
            Ok(T::from(&bytes[1..]))
        } else {
            T::deserialize(deserializer)
        }
        .map(Self)
    }
}

#[cfg(feature = "hashing")]
impl<T: mina_hasher::Hashable, U, const V: u8> mina_hasher::Hashable
    for Base58CheckOfBinProt<T, U, V>
{
    type D = T::D;

    fn to_roinput(&self) -> mina_hasher::ROInput {
        self.0.to_roinput()
    }

    fn domain_string(domain_param: Self::D) -> Option<String> {
        T::domain_string(domain_param)
    }
}

#[cfg(feature = "hashing")]
impl<T: mina_hasher::Hashable, const V: u8> mina_hasher::Hashable for Base58CheckOfBytes<T, V> {
    type D = T::D;

    fn to_roinput(&self) -> mina_hasher::ROInput {
        self.0.to_roinput()
    }

    fn domain_string(domain_param: Self::D) -> Option<String> {
        T::domain_string(domain_param)
    }
}

#[cfg(test)]
mod tests {
    use super::ToBase58Check;
    use binprot::BinProtRead;
    use binprot_derive::{BinProtRead, BinProtWrite};
    use serde::{Deserialize, Serialize};

    use crate::{
        b58::{Base58CheckOfBinProt, Base58CheckVersion},
        bigint::BigInt,
        versioned::Versioned,
    };

    macro_rules! base58tests {
        ($($name:ident($bytes:expr, $version:expr, $b58:expr $(,)? )),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let bytes = hex::decode($bytes).unwrap();

                    let encoded = super::encode(&bytes, $version);
                    assert_eq!(&encoded, $b58);

                    let decoded = super::decode($b58, $version).unwrap();
                    assert_eq!(decoded[0], $version);
                    assert_eq!(&decoded[1..], &bytes);
                }
            )*
        };
    }

    base58tests!(
        genesis_state_hash(
            "01fc630629c6a1a237a3dc1d95fd54fbf9cca062486e9f57852ebc64e4042ceb3d",
            0x10,
            "3NLx3eBDTvYmP27bUmYANzmhjL5rGe36nGW6N5XhGcuStF6Zv7ZD"
        ),
        prev_state_hash(
            "019b4f7c30bcf6c883c388097db490bfeeae5a1d36eb4b593af65e3511db4fc432",
            0x10,
            "3NLDHxUsL6Ehujf6x2AT6CXrpRjeY1rw9e93QJfJUAD3P6HVVtcA"
        ),
        state_hash(
            "018d67aadd018581a812623915b13d5c3a6da7dfe8a195172d9bbd206810bc2329",
            0x10,
            "3NL7AkynW6hbDrhHTAht1GLG563Fo9fdcEQk1zEyy5XedC6aZTeB",
        ),
        ledger_hash(
            "01636f5b2d67278e17bc4343c7c23fb4991f8cf0bbbfd8558615b124d5d6254801",
            0x05,
            "jwrPvAMUNo3EKT2puUk5Fxz6B7apRAoKNTGpAA49t3TRSfzvdrL"
        ),
        address(
            "01013c2b5b48c22dc8b8c9d2c9d76a2ceaaf02beabb364301726c3f8e989653af51300",
            0xcb,
            "B62qkUHaJUHERZuCHQhXCQ8xsGBqyYSgjQsKnKN5HhSJecakuJ4pYyk"
        )
    );

    fn bigint(s: &str) -> BigInt {
        let bytes = hex::decode(s).unwrap();
        BigInt::binprot_read(&mut &bytes[..]).unwrap()
    }

    #[test]
    #[ignore = "fix or remove"]
    fn binable_base58check() {
        #[derive(Clone, BinProtRead, BinProtWrite)]
        struct Binable(BigInt);

        impl Base58CheckVersion for Binable {
            const VERSION_BYTE: u8 = 0x10;
        }

        let b = Binable(bigint(
            "fc630629c6a1a237a3dc1d95fd54fbf9cca062486e9f57852ebc64e4042ceb3d",
        ));
        let b58c = b.to_base58check().unwrap();
        assert_eq!(
            &b58c,
            "3NLx3eBDTvYmP27bUmYANzmhjL5rGe36nGW6N5XhGcuStF6Zv7ZD"
        )
    }

    #[test]
    fn serde_as_base58check() {
        #[derive(Clone, Debug, BinProtRead, BinProtWrite, Serialize, Deserialize, PartialEq)]
        struct BinableV1(BigInt);

        impl Base58CheckVersion for BinableV1 {
            const VERSION_BYTE: u8 = 0x10;
        }

        #[derive(Clone, Debug, BinProtRead, BinProtWrite, Serialize, Deserialize, PartialEq)]
        struct BinableV2(BigInt);

        impl From<BinableV2> for Versioned<BinableV1, 1> {
            fn from(v2: BinableV2) -> Self {
                BinableV1(v2.0).into()
            }
        }

        impl From<Versioned<BinableV1, 1>> for BinableV2 {
            fn from(v1: Versioned<BinableV1, 1>) -> Self {
                BinableV2(v1.into_inner().0)
            }
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Foo {
            b: Base58CheckOfBinProt<BinableV2, Versioned<BinableV1, 1>, 0x10>,
        }

        let b = Foo {
            b: BinableV2(bigint(
                "fc630629c6a1a237a3dc1d95fd54fbf9cca062486e9f57852ebc64e4042ceb3d",
            ))
            .into(),
        };
        assert_eq!(
            &serde_json::to_string(&b).unwrap(),
            r#"{"b":"3NLx3eBDTvYmP27bUmYANzmhjL5rGe36nGW6N5XhGcuStF6Zv7ZD"}"#
        );
        assert_eq!(
            serde_json::from_str::<Foo>(
                r#"{"b":"3NLx3eBDTvYmP27bUmYANzmhjL5rGe36nGW6N5XhGcuStF6Zv7ZD"}"#
            )
            .unwrap(),
            b
        );
    }
}

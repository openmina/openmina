use binprot::Nat0;
use serde::{de::Visitor, Deserialize, Serialize};

const MINA_STRING_MAX_LENGTH: usize = 100_000_000;
const CHUNK_SIZE: usize = 5_000;

/// String of bytes.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteString(pub Vec<u8>);

impl std::fmt::Debug for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        // Avoid vertical alignment
        f.write_fmt(format_args!("ByteString({:?})", inner))
    }
}

impl std::ops::Deref for ByteString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for ByteString {
    fn from(source: Vec<u8>) -> Self {
        Self(source)
    }
}

impl From<&[u8]> for ByteString {
    fn from(source: &[u8]) -> Self {
        Self(source.to_vec())
    }
}

impl From<&str> for ByteString {
    fn from(source: &str) -> Self {
        Self(source.as_bytes().to_vec())
    }
}

impl TryFrom<ByteString> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: ByteString) -> Result<Self, Self::Error> {
        String::from_utf8(value.0)
    }
}

impl TryFrom<&ByteString> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: &ByteString) -> Result<Self, Self::Error> {
        String::from_utf8(value.0.clone())
    }
}

impl Serialize for ByteString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !serializer.is_human_readable() {
            return self.0.serialize(serializer);
        }
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for ByteString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !deserializer.is_human_readable() {
            return Vec::<u8>::deserialize(deserializer).map(Self);
        }
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Vec<u8>;

            fn expecting(
                &self,
                formatter: &mut serde::__private::fmt::Formatter,
            ) -> serde::__private::fmt::Result {
                formatter.write_str("hex string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                hex::decode(v)
                    .map_err(|_| serde::de::Error::custom("failed to decode hex str".to_string()))
            }
        }
        deserializer.deserialize_str(V).map(Self)
    }
}

impl binprot::BinProtRead for ByteString {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let len = Nat0::binprot_read(r)?.0 as usize;
        if len > MINA_STRING_MAX_LENGTH {
            return Err(MinaStringTooLong::as_binprot_err(len));
        }

        Ok(Self(maybe_read_in_chunks(len, r)?))
    }
}

impl binprot::BinProtWrite for ByteString {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        if self.0.len() > MINA_STRING_MAX_LENGTH {
            return Err(MinaStringTooLong::as_io_err(self.0.len()));
        }
        Nat0(self.0.len() as u64).binprot_write(w)?;
        w.write_all(&self.0)?;
        Ok(())
    }
}

/// Human-readable string.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct CharString(Vec<u8>);

impl std::fmt::Debug for CharString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        // Avoid vertical alignment
        f.write_fmt(format_args!("CharString({:?})", inner))
    }
}

impl CharString {
    pub fn to_string_lossy(&self) -> std::string::String {
        std::string::String::from_utf8_lossy(&self.0).into_owned()
    }
}

impl AsRef<[u8]> for CharString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Vec<u8>> for CharString {
    fn from(source: Vec<u8>) -> Self {
        Self(source)
    }
}

impl From<&[u8]> for CharString {
    fn from(source: &[u8]) -> Self {
        Self(source.to_vec())
    }
}

impl From<&str> for CharString {
    fn from(source: &str) -> Self {
        Self(source.as_bytes().to_vec())
    }
}

impl PartialEq<[u8]> for CharString {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<str> for CharString {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other.as_bytes()
    }
}

impl std::fmt::Display for CharString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string_lossy())
    }
}

impl Serialize for CharString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !serializer.is_human_readable() {
            return self.0.serialize(serializer);
        }
        let s = match std::string::String::from_utf8(self.0.clone()) {
            Ok(s) => s,
            Err(e) => return Err(serde::ser::Error::custom(format!("{e}"))),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for CharString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !deserializer.is_human_readable() {
            return Vec::<u8>::deserialize(deserializer).map(Self);
        }
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Vec<u8>;

            fn expecting(
                &self,
                formatter: &mut serde::__private::fmt::Formatter,
            ) -> serde::__private::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.as_bytes().to_vec())
            }
        }
        deserializer.deserialize_str(V).map(Self)
    }
}

impl binprot::BinProtRead for CharString {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let len = Nat0::binprot_read(r)?.0 as usize;
        if len > MINA_STRING_MAX_LENGTH {
            return Err(MinaStringTooLong::as_binprot_err(len));
        }

        Ok(Self(maybe_read_in_chunks(len, r)?))
    }
}

impl binprot::BinProtWrite for CharString {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        if self.0.len() > MINA_STRING_MAX_LENGTH {
            return Err(MinaStringTooLong::as_io_err(self.0.len()));
        }
        Nat0(self.0.len() as u64).binprot_write(w)?;
        w.write_all(&self.0)?;
        Ok(())
    }
}

/// Reads data from the reader `r` in chunks if the length `len` exceeds a predefined chunk size.
///
/// This approach avoids preallocating a large buffer upfront, which is crucial for handling
/// potentially large or untrusted input sizes efficiently and safely.
fn maybe_read_in_chunks<R: std::io::Read + ?Sized>(
    len: usize,
    r: &mut R,
) -> Result<Vec<u8>, binprot::Error> {
    if len <= CHUNK_SIZE {
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        Ok(buf)
    } else {
        let mut buf = vec![0u8; CHUNK_SIZE];
        let mut temp_buf = vec![0u8; CHUNK_SIZE];
        let mut remaining = len;
        while remaining > 0 {
            let read_size = std::cmp::min(CHUNK_SIZE, remaining);
            r.read_exact(&mut temp_buf[..read_size])?;
            buf.extend_from_slice(&temp_buf[..read_size]);
            remaining -= read_size;
        }
        Ok(buf)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("String length `{actual}` is greater than maximum `{max}`")]
pub struct MinaStringTooLong {
    max: usize,
    actual: usize,
}

impl MinaStringTooLong {
    fn boxed(actual: usize) -> Box<Self> {
        Box::new(MinaStringTooLong {
            max: MINA_STRING_MAX_LENGTH,
            actual,
        })
    }

    fn as_io_err(actual: usize) -> std::io::Error {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            MinaStringTooLong::boxed(actual),
        )
    }

    fn as_binprot_err(actual: usize) -> binprot::Error {
        binprot::Error::CustomError(MinaStringTooLong::boxed(actual))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binprot::{BinProtRead, BinProtWrite, Nat0};

    use crate::string::CharString;

    use super::{ByteString, MINA_STRING_MAX_LENGTH};

    #[test]
    fn bounded_string_binprot_write() {
        let bs = ByteString::from(vec![0; MINA_STRING_MAX_LENGTH]);
        let mut out = Vec::new();
        let res = bs.binprot_write(&mut out);
        assert!(res.is_ok());

        let bs = CharString::from(vec![0; MINA_STRING_MAX_LENGTH]);
        let mut out = Vec::new();
        let res = bs.binprot_write(&mut out);
        assert!(res.is_ok());

        let bs = ByteString::from(vec![0; MINA_STRING_MAX_LENGTH + 1]);
        let mut out = Vec::new();
        let res = bs.binprot_write(&mut out);
        assert!(res.is_err());

        let bs = CharString::from(vec![0; MINA_STRING_MAX_LENGTH + 1]);
        let mut out = Vec::new();
        let res = bs.binprot_write(&mut out);
        assert!(res.is_err());
    }

    #[test]
    fn bounded_string_binprot_read() {
        fn input(len: usize) -> Cursor<Vec<u8>> {
            let mut input = Vec::new();
            // prepare input
            Nat0(len as u64).binprot_write(&mut input).unwrap();
            vec![0; len].binprot_write(&mut input).unwrap();
            Cursor::new(input)
        }

        let mut inp = input(MINA_STRING_MAX_LENGTH);
        let res = ByteString::binprot_read(&mut inp);
        assert!(res.is_ok());

        let mut inp = input(MINA_STRING_MAX_LENGTH);
        let res = CharString::binprot_read(&mut inp);
        assert!(res.is_ok());

        let mut inp = input(MINA_STRING_MAX_LENGTH + 1);
        let res = ByteString::binprot_read(&mut inp);
        assert!(res.is_err());

        let mut inp = input(MINA_STRING_MAX_LENGTH + 1);
        let res = CharString::binprot_read(&mut inp);
        assert!(res.is_err());
    }
}

// Limited length

// TODO(tizoc): should probably redefine the above types using LimitedLength versions

use std::marker::PhantomData;

/// A string-like type limited to a maximum byte length.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct LimitedLengthString<const MAX_LENGTH: usize>(Vec<u8>, PhantomData<[u8; MAX_LENGTH]>);

impl<const MAX_LENGTH: usize> std::fmt::Debug for LimitedLengthString<MAX_LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner, _) = self;
        f.write_fmt(format_args!("LimitedLengthString({:?})", inner))
    }
}

impl<const MAX_LENGTH: usize> LimitedLengthString<MAX_LENGTH> {
    pub fn to_string_lossy(&self) -> std::string::String {
        std::string::String::from_utf8_lossy(&self.0).into_owned()
    }
}

impl<const MAX_LENGTH: usize> AsRef<[u8]> for LimitedLengthString<MAX_LENGTH> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<const MAX_LENGTH: usize> From<Vec<u8>> for LimitedLengthString<MAX_LENGTH> {
    fn from(source: Vec<u8>) -> Self {
        Self(source, PhantomData)
    }
}

impl<const MAX_LENGTH: usize> From<&[u8]> for LimitedLengthString<MAX_LENGTH> {
    fn from(source: &[u8]) -> Self {
        Self(source.to_vec(), PhantomData)
    }
}

impl<const MAX_LENGTH: usize> From<&str> for LimitedLengthString<MAX_LENGTH> {
    fn from(source: &str) -> Self {
        Self(source.as_bytes().to_vec(), PhantomData)
    }
}

impl<const MAX_LENGTH: usize> TryFrom<LimitedLengthString<MAX_LENGTH>> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: LimitedLengthString<MAX_LENGTH>) -> Result<Self, Self::Error> {
        String::from_utf8(value.0)
    }
}

impl<const MAX_LENGTH: usize> TryFrom<&LimitedLengthString<MAX_LENGTH>> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: &LimitedLengthString<MAX_LENGTH>) -> Result<Self, Self::Error> {
        String::from_utf8(value.0.clone())
    }
}

impl<const MAX_LENGTH: usize> std::fmt::Display for LimitedLengthString<MAX_LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string_lossy())
    }
}

impl<const MAX_LENGTH: usize> Serialize for LimitedLengthString<MAX_LENGTH> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.0.len() > MAX_LENGTH {
            return Err(serde::ser::Error::custom(format!(
                "LimitedLengthString exceeds max length of {}",
                MAX_LENGTH
            )));
        }

        if !serializer.is_human_readable() {
            return self.0.serialize(serializer);
        }
        let s = match std::string::String::from_utf8(self.0.clone()) {
            Ok(s) => s,
            Err(e) => return Err(serde::ser::Error::custom(format!("{e}"))),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de, const MAX_LENGTH: usize> Deserialize<'de> for LimitedLengthString<MAX_LENGTH> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !deserializer.is_human_readable() {
            return Vec::<u8>::deserialize(deserializer).map(|v| Self(v, PhantomData));
        }
        struct V<const MAX_LENGTH: usize>;

        impl<'de, const MAX_LENGTH: usize> Visitor<'de> for V<MAX_LENGTH> {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let bytes = v.as_bytes().to_vec();
                if bytes.len() > MAX_LENGTH {
                    return Err(E::custom(format!(
                        "LimitedLengthString exceeds max length: {}",
                        MAX_LENGTH
                    )));
                }
                Ok(bytes)
            }
        }

        deserializer
            .deserialize_str(V::<MAX_LENGTH>)
            .map(|v| Self(v, PhantomData))
    }
}

impl<const MAX_LENGTH: usize> binprot::BinProtRead for LimitedLengthString<MAX_LENGTH> {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let len = Nat0::binprot_read(r)?.0 as usize;
        if len > MAX_LENGTH {
            return Err(LimitedLengthStringTooLong::as_binprot_err(len, MAX_LENGTH));
        }

        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        Ok(Self(buf, PhantomData))
    }
}

impl<const MAX_LENGTH: usize> binprot::BinProtWrite for LimitedLengthString<MAX_LENGTH> {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        if self.0.len() > MAX_LENGTH {
            return Err(LimitedLengthStringTooLong::as_io_err(
                self.0.len(),
                MAX_LENGTH,
            ));
        }
        Nat0(self.0.len() as u64).binprot_write(w)?;
        w.write_all(&self.0)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("LimitedLengthString length `{actual}` is greater than maximum `{max}`")]
pub struct LimitedLengthStringTooLong {
    max: usize,
    actual: usize,
}

impl LimitedLengthStringTooLong {
    fn boxed(actual: usize, max: usize) -> Box<Self> {
        Box::new(LimitedLengthStringTooLong { max, actual })
    }

    fn as_io_err(actual: usize, max: usize) -> std::io::Error {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            LimitedLengthStringTooLong::boxed(actual, max),
        )
    }

    fn as_binprot_err(actual: usize, max: usize) -> binprot::Error {
        binprot::Error::CustomError(LimitedLengthStringTooLong::boxed(actual, max))
    }
}

// https://github.com/MinaProtocol/mina/blob/c0c9d702b8cba34a603a28001c293ca462b1dfec/src/lib/mina_base/zkapp_account.ml#L140
pub const ZKAPP_URI_MAX_LENGTH: usize = 255;
// https://github.com/MinaProtocol/mina/blob/c0c9d702b8cba34a603a28001c293ca462b1dfec/src/lib/mina_base/account.ml#L92
pub const TOKEN_SYMBOL_MAX_LENGTH: usize = 6;

pub type ZkAppUri = LimitedLengthString<ZKAPP_URI_MAX_LENGTH>;
pub type TokenSymbol = LimitedLengthString<TOKEN_SYMBOL_MAX_LENGTH>;

#[cfg(test)]
mod limited_length_string_tests {
    use super::{ZkAppUri, ZKAPP_URI_MAX_LENGTH};
    use binprot::{BinProtRead, BinProtWrite, Nat0};
    use std::io::Cursor;

    #[test]
    fn limited_length_string_serialize_deserialize() {
        let valid_str = "a".repeat(ZKAPP_URI_MAX_LENGTH); // max-length string
        let valid_uri = ZkAppUri::from(valid_str.as_str());
        let serialized = serde_json::to_string(&valid_uri).unwrap();
        let deserialized: ZkAppUri = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.to_string_lossy(), valid_str);

        let invalid_str = "a".repeat(ZKAPP_URI_MAX_LENGTH + 1); // exceeding max-length string
        let invalid_uri = ZkAppUri::from(invalid_str.as_str());
        let result = serde_json::to_string(&invalid_uri);
        assert!(
            result.is_err(),
            "Expected serialization to fail for string longer than 255 bytes"
        );

        let invalid_json = format!("\"{}\"", "a".repeat(ZKAPP_URI_MAX_LENGTH + 1));
        let deserialization_result: Result<ZkAppUri, _> = serde_json::from_str(&invalid_json);
        assert!(
            deserialization_result.is_err(),
            "Expected deserialization to fail for string longer than 255 bytes"
        );
    }

    #[test]
    fn limited_length_string_binprot_write() {
        let valid_uri = ZkAppUri::from(vec![0; ZKAPP_URI_MAX_LENGTH]);
        let mut out = Vec::new();
        let res = valid_uri.binprot_write(&mut out);
        assert!(res.is_ok());

        let invalid_uri = ZkAppUri::from(vec![0; ZKAPP_URI_MAX_LENGTH + 1]);
        let mut out = Vec::new();
        let res = invalid_uri.binprot_write(&mut out);
        assert!(res.is_err());
    }

    #[test]
    fn limited_length_string_binprot_read() {
        fn input(len: usize) -> Cursor<Vec<u8>> {
            let mut input = Vec::new();
            Nat0(len as u64).binprot_write(&mut input).unwrap();
            vec![0; len].binprot_write(&mut input).unwrap();
            Cursor::new(input)
        }

        let mut inp = input(ZKAPP_URI_MAX_LENGTH);
        let res = ZkAppUri::binprot_read(&mut inp);
        assert!(res.is_ok());

        let mut inp = input(ZKAPP_URI_MAX_LENGTH + 1);
        let res = ZkAppUri::binprot_read(&mut inp);
        assert!(res.is_err());
    }
}

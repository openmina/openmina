use binprot::Nat0;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_bytes;

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
        let s = self
            .0
            .iter()
            .map(|&byte| {
                if byte.is_ascii_graphic() {
                    (byte as char).to_string()
                } else {
                    // Convert non-printable bytes to escape sequences
                    format!("\\x{:02x}", byte)
                }
            })
            .collect::<String>();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for ByteString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        if !deserializer.is_human_readable() {
            return Vec::<u8>::deserialize(deserializer).map(Self);
        }
        let s: serde_bytes::ByteBuf = Deserialize::deserialize(deserializer)?;
        Ok(s.into_vec().into())
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

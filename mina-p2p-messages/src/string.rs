use binprot::Nat0;
use serde::{de::Visitor, Deserialize, Serialize};

/// String of bytes.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteString(Vec<u8>);

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
                    .map_err(|_| serde::de::Error::custom(format!("failed to decode hex str")))
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
        let len = Nat0::binprot_read(r)?;
        let mut buf: Vec<u8> = vec![0u8; len.0 as usize];
        r.read_exact(&mut buf)?;
        Ok(Self(buf))
    }
}

impl binprot::BinProtWrite for ByteString {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let _ = Nat0(self.0.len() as u64).binprot_write(w)?;
        let _ = w.write_all(&self.0)?;
        Ok(())
    }
}

/// Human-readable string.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharString(Vec<u8>);

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
        let len = Nat0::binprot_read(r)?;
        let mut buf: Vec<u8> = vec![0u8; len.0 as usize];
        r.read_exact(&mut buf)?;
        Ok(Self(buf))
    }
}

impl binprot::BinProtWrite for CharString {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let _ = Nat0(self.0.len() as u64).binprot_write(w)?;
        let _ = w.write_all(&self.0)?;
        Ok(())
    }
}

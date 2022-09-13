use binprot::Nat0;
use serde::{Deserialize, Serialize};

/// String of bytes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct String(Vec<u8>);

impl AsRef<[u8]> for String {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&str> for String {
    fn from(source: &str) -> Self {
        Self(source.as_bytes().to_vec())
    }
}

impl Serialize for String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for String {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl binprot::BinProtRead for String {
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

impl binprot::BinProtWrite for String {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let _ = Nat0(self.0.len() as u64).binprot_write(w)?;
        let _ = w.write_all(&self.0)?;
        Ok(())
    }
}

/// Human-readable string.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharString(std::string::String);

impl AsRef<str> for CharString {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<&str> for CharString {
    fn from(source: &str) -> Self {
        Self(source.to_string())
    }
}

impl Serialize for CharString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CharString {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
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
        match std::string::String::from_utf8(buf) {
            Ok(s) => Ok(Self(s)),
            Err(err) => Err(binprot::Error::CustomError(Box::new(err))),
        }
    }
}

impl binprot::BinProtWrite for CharString {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let bytes = self.0.as_bytes();
        let _ = Nat0(bytes.len() as u64).binprot_write(w)?;
        let _ = w.write_all(bytes)?;
        Ok(())
    }
}

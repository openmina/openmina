use std::{
    fmt,
    io::{Read, Write},
    str::FromStr,
};

use binprot::{BinProtRead, BinProtWrite};
use ed25519_dalek::Signature as Ed25519Signature;
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Serialize,
};

#[derive(Eq, PartialEq, Clone)]
pub struct Signature(pub(super) Ed25519Signature);

impl Signature {
    const BYTE_SIZE: usize = Ed25519Signature::BYTE_SIZE;

    pub fn from_bytes(bytes: [u8; Self::BYTE_SIZE]) -> Self {
        Self(Ed25519Signature::from_bytes(&bytes))
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTE_SIZE] {
        self.0.to_bytes()
    }
}

impl FromStr for Signature {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex::decode(s)?
            .try_into()
            .map(Self::from_bytes)
            .or(Err(hex::FromHexError::InvalidStringLength))
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_bytes()))
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({self})")
    }
}

impl From<Signature> for [u8; Signature::BYTE_SIZE] {
    fn from(value: Signature) -> Self {
        value.to_bytes()
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            self.to_bytes().serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s: String = Deserialize::deserialize(deserializer)?;
            Ok(s.parse().map_err(serde::de::Error::custom)?)
        } else {
            struct ArrayVisitor;

            impl<'de> Visitor<'de> for ArrayVisitor {
                type Value = [u8; Signature::BYTE_SIZE];

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "signature bytes({})", Signature::BYTE_SIZE)
                }

                #[inline]
                fn visit_seq<A>(self, mut a: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut bytes: Self::Value = [0; Signature::BYTE_SIZE];

                    for (i, byte) in bytes.iter_mut().enumerate() {
                        *byte = a
                            .next_element()?
                            .ok_or(serde::de::Error::invalid_length(i + 1, &self))?;
                    }

                    Ok(bytes)
                }
            }

            deserializer
                .deserialize_tuple(Self::BYTE_SIZE, ArrayVisitor)
                .map(Self::from_bytes)
        }
    }
}

impl BinProtWrite for Signature {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.to_bytes())
    }
}

impl BinProtRead for Signature {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let mut buf = [0; Ed25519Signature::BYTE_SIZE];
        r.read_exact(&mut buf)?;
        Ok(Self::from_bytes(buf))
    }
}

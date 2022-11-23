use binprot::byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Char(pub u8);

impl std::ops::Deref for Char {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Char {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl Serialize for Char {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            char::from_u32(self.0.into())
                .unwrap()
                .to_string()
                .serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Char {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn str_to_char(v: &str) -> Result<u8, &str> {
            if v.len() != 1 {
                return Err("incorrect length");
            }
            let ch = v.chars().next().unwrap();
            (ch as u32).try_into().map_err(|_| "incorrect char")
        }
        if deserializer.is_human_readable() {
            struct V;
            impl<'de> Visitor<'de> for V {
                type Value = u8;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("string consisting of a single escaped unicode character")
                }

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    str_to_char(&v).map_err(serde::de::Error::custom)
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    str_to_char(v).map_err(serde::de::Error::custom)
                }
            }

            deserializer.deserialize_string(V)
        } else {
            u8::deserialize(deserializer)
        }
        .map(Self)
    }
}

impl From<u8> for Char {
    fn from(source: u8) -> Self {
        Self(source)
    }
}

impl From<Char> for u8 {
    fn from(source: Char) -> Self {
        source.0
    }
}

impl binprot::BinProtRead for Char {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        Ok(r.read_u8().map(Self)?)
    }
}

impl binprot::BinProtWrite for Char {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_u8(self.0)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn json() {
        let ch = super::Char(16);
        let json = serde_json::to_value(&ch).unwrap();
        assert_eq!(json.as_str().unwrap(), "\u{0010}");
        assert_eq!(
            serde_json::from_value::<super::Char>(json).unwrap(),
            super::Char(16)
        );
    }
}

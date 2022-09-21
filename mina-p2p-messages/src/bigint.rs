use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
pub struct BigInt([u8; 32]);

impl binprot::BinProtRead for BigInt {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let mut buf = [0; 32];
        r.read_exact(&mut buf)?;
        Ok(Self(buf))
    }
}

impl binprot::BinProtWrite for BigInt {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.0)?;
        Ok(())
    }
}

impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(&self.0))
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for BigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            struct V;
            impl<'de> serde::de::Visitor<'de> for V {
                type Value = &'de str;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("hex string")
                }

                fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(v)
                }
            }
            let s: &str = deserializer.deserialize_str(V)?;
            let v = hex::decode(s)
                .map_err(|_| serde::de::Error::custom(format!("failed to decode hex str")))?;
            v.try_into()
                .map_err(|_| serde::de::Error::custom(format!("failed to convert vec to array")))
                .map(Self)
        } else {
            struct V;
            impl<'de> serde::de::Visitor<'de> for V {
                type Value = [u8; 32];

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("sequence of 32 bytes")
                }

                fn visit_borrowed_bytes<E>(self, _v: &'de [u8]) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    todo!()
                }
            }
            deserializer.deserialize_bytes(V).map(Self)
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::BigInt;

    fn from_byte(b: u8) -> BigInt {
        BigInt([b; 32])
    }

    fn from_bytes<'a, I>(it: I) -> BigInt
    where
        I: IntoIterator<Item = &'a u8>,
        I::IntoIter: Clone,
    {
        let mut bytes = [0; 32];
        let it = it.into_iter().cycle();
        bytes.iter_mut().zip(it).for_each(|(b, i)| *b = *i);
        BigInt(bytes)
    }

    #[test]
    fn serialize_bigint() {
        let bigints = [
            from_byte(0),
            from_byte(1),
            from_byte(0xff),
            from_bytes(&[0, 1, 2, 3, 4]),
        ];

        for bigint in bigints {
            let binprot = serde_binprot::to_vec(&bigint).unwrap();
            assert_eq!(binprot.as_slice(), &bigint.0);
        }
    }

    #[test]
    fn deserialize_bigint() {
        let bigints = [
            from_byte(0),
            from_byte(1),
            from_byte(0xff),
            from_bytes(&[0, 1, 2, 3, 4]),
        ];

        for bigint in bigints {
            let deser: BigInt = serde_binprot::from_slice(&bigint.0).unwrap();
            assert_eq!(&bigint.0, &deser.0);
        }
    }
}
*/

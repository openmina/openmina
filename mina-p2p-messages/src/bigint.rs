use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, derive_more::From, derive_more::Into)]
pub struct BigInt(Box<[u8; 32]>);

impl BigInt {
    #[cfg(feature = "hashing")]
    pub fn to_fp(&self) -> Result<mina_hasher::Fp, o1_utils::field_helpers::FieldHelpersError> {
        use o1_utils::FieldHelpers;
        mina_hasher::Fp::from_bytes(self.0.as_ref())
    }

    pub fn iter_bytes<'a>(&'a self) -> impl 'a + DoubleEndedIterator<Item = u8> {
        self.0.iter().cloned()
    }
}

impl AsRef<[u8]> for BigInt {
    fn as_ref(&self) -> &[u8] {
        &self.0.as_ref()[..]
    }
}

#[cfg(feature = "hashing")]
impl From<mina_curves::pasta::Fp> for BigInt {
    fn from(field: mina_curves::pasta::Fp) -> Self {
        use o1_utils::FieldHelpers;
        Self(Box::new(field.to_bytes().try_into().unwrap()))
    }
}

#[cfg(feature = "hashing")]
impl From<mina_curves::pasta::Fq> for BigInt {
    fn from(field: mina_curves::pasta::Fq) -> Self {
        use o1_utils::FieldHelpers;
        Self(Box::new(field.to_bytes().try_into().unwrap()))
    }
}

impl binprot::BinProtRead for BigInt {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let mut buf = [0; 32];
        r.read_exact(&mut buf)?;
        Ok(Self(Box::new(buf)))
    }
}

impl binprot::BinProtWrite for BigInt {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.0[..])?;
        Ok(())
    }
}

impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(&self.0[..]))
        } else {
            serializer.serialize_bytes(&self.0[..])
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
                type Value = Vec<u8>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("hex string")
                }

                fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    hex::decode(v)
                        .map_err(|_| serde::de::Error::custom(format!("failed to decode hex str")))
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    hex::decode(v)
                        .map_err(|_| serde::de::Error::custom(format!("failed to decode hex str")))
                }
            }
            let v = deserializer.deserialize_str(V)?;
            v.try_into()
                .map_err(|_| serde::de::Error::custom(format!("failed to convert vec to array")))
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
            deserializer.deserialize_bytes(V)
        }
        .map(Box::new)
        .map(Self)
    }
}

#[cfg(feature = "hashing")]
impl mina_hasher::Hashable for BigInt {
    type D = ();

    fn to_roinput(&self) -> mina_hasher::ROInput {
        mina_hasher::ROInput::new()
            .append_field(self.to_fp().expect("Failed to convert Hash into Fp"))
    }

    fn domain_string(_: Self::D) -> Option<String> {
        None
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

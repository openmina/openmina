use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::From, derive_more::Into)]
pub struct BigInt(Box<[u8; 32]>);

impl std::fmt::Debug for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        // Avoid vertical alignment
        f.write_fmt(format_args!("BigInt({:?})", inner))
    }
}

impl BigInt {
    #[cfg(feature = "hashing")]
    pub fn zero() -> Self {
        mina_curves::pasta::Fp::from(0u64).into()
    }

    #[cfg(feature = "hashing")]
    pub fn one() -> Self {
        mina_curves::pasta::Fp::from(1u64).into()
    }

    #[cfg(feature = "hashing")]
    pub fn to_fp(&self) -> Result<mina_hasher::Fp, o1_utils::field_helpers::FieldHelpersError> {
        use o1_utils::FieldHelpers;
        mina_hasher::Fp::from_bytes(self.0.as_ref())
    }

    #[cfg(feature = "hashing")]
    pub fn to_field<F>(&self) -> F
    where
        F: ark_ff::Field,
    {
        let mut slice: &[u8] = self.0.as_ref();
        F::read(&mut slice).expect("Conversion BigInt to Field failed")
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

#[cfg(feature = "hashing")]
impl From<&mina_curves::pasta::Fp> for BigInt {
    fn from(field: &mina_curves::pasta::Fp) -> Self {
        use o1_utils::FieldHelpers;
        Self(Box::new(field.to_bytes().try_into().unwrap()))
    }
}

#[cfg(feature = "hashing")]
impl From<&mina_curves::pasta::Fq> for BigInt {
    fn from(field: &mina_curves::pasta::Fq) -> Self {
        use o1_utils::FieldHelpers;
        Self(Box::new(field.to_bytes().try_into().unwrap()))
    }
}

#[cfg(feature = "hashing")]
impl From<BigInt> for mina_curves::pasta::Fp {
    fn from(bigint: BigInt) -> Self {
        bigint.to_field()
    }
}

#[cfg(feature = "hashing")]
impl From<BigInt> for mina_curves::pasta::Fq {
    fn from(bigint: BigInt) -> Self {
        bigint.to_field()
    }
}

#[cfg(feature = "hashing")]
impl From<&BigInt> for mina_curves::pasta::Fp {
    fn from(bigint: &BigInt) -> Self {
        bigint.to_field()
    }
}

#[cfg(feature = "hashing")]
impl From<&BigInt> for mina_curves::pasta::Fq {
    fn from(bigint: &BigInt) -> Self {
        bigint.to_field()
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
            // TODO get rid of copying
            let mut rev = self.0.as_ref().clone();
            rev[..].reverse();
            let mut hex = [0_u8; 32 * 2 + 2];
            hex[..2].copy_from_slice(b"0x");
            hex::encode_to_slice(rev, &mut hex[2..]).unwrap();
            serializer.serialize_str(String::from_utf8_lossy(&hex).as_ref())
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
                    match v.strip_prefix("0x") {
                        Some(v) => hex::decode(v).map_err(|_| {
                            serde::de::Error::custom(format!("failed to decode hex str: {v}"))
                        }),
                        None => Err(serde::de::Error::custom(format!("mising 0x prefix"))),
                    }
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match v.strip_prefix("0x") {
                        Some(v) => hex::decode(v).map_err(|_| {
                            serde::de::Error::custom(format!("failed to decode hex str: {v}"))
                        }),
                        None => Err(serde::de::Error::custom(format!("mising 0x prefix"))),
                    }
                }
            }
            let mut v = deserializer.deserialize_str(V)?;
            v.reverse();
            v.try_into()
                .map_err(|_| serde::de::Error::custom(format!("failed to convert vec to array")))
        } else {
            struct V;
            impl<'de> serde::de::Visitor<'de> for V {
                type Value = [u8; 32];

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("sequence of 32 bytes")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    let v: [u8; 32] = v
                        .try_into()
                        .map_err(|_| serde::de::Error::custom("expecting 32 bytes".to_string()))?;
                    Ok(v)
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

#[cfg(test)]
mod tests {
    use binprot::{BinProtRead, BinProtWrite};

    use super::BigInt;

    fn to_binprot(v: &BigInt) -> Vec<u8> {
        let mut w = Vec::new();
        v.binprot_write(&mut w).unwrap();
        w
    }

    fn from_binprot(mut b: &[u8]) -> BigInt {
        BigInt::binprot_read(&mut b).unwrap()
    }

    fn from_byte(b: u8) -> BigInt {
        BigInt(Box::new([b; 32]))
    }

    fn from_bytes<'a, I>(it: I) -> BigInt
    where
        I: IntoIterator<Item = &'a u8>,
        I::IntoIter: Clone,
    {
        let mut bytes = [0; 32];
        let it = it.into_iter().cycle();
        bytes.iter_mut().zip(it).for_each(|(b, i)| *b = *i);
        BigInt(Box::new(bytes))
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
            let binprot = to_binprot(&bigint);
            assert_eq!(binprot.as_slice(), bigint.0.as_ref());
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
            let deser: BigInt = from_binprot(bigint.0.as_ref());
            assert_eq!(&bigint.0, &deser.0);
        }
    }

    #[test]
    fn to_json() {
        let bigints = [
            from_byte(0),
            from_byte(1),
            from_byte(0xff),
            from_bytes(&[0, 1, 2, 3, 4]),
        ];

        for bigint in bigints {
            let json = serde_json::to_string(&bigint).unwrap();
            let mut v = bigint.0.as_ref().clone();
            v.reverse();
            let json_exp = format!(r#""0x{}""#, hex::encode(v));
            assert_eq!(json, json_exp);
        }
    }

    #[test]
    fn from_json() {
        let bigints = [
            from_byte(0),
            from_byte(1),
            from_byte(0xff),
            from_bytes(&[0, 1, 2, 3, 4]),
        ];

        for bigint in bigints {
            let mut be = bigint.0.clone();
            be.reverse();
            let json = format!(r#""0x{}""#, hex::encode(be.as_ref()));
            let bigint_exp = serde_json::from_str(&json).unwrap();
            assert_eq!(bigint, bigint_exp);
        }
    }
}

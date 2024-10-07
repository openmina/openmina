use ark_ff::{fields::arithmetic::InvalidBigInt, BigInteger256};
use rsexp::{OfSexp, SexpOf};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::From, derive_more::Into)]
pub struct BigInt(BigInteger256);

impl std::fmt::Debug for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(BigInteger256(array)) = self;
        // Avoid vertical alignment
        f.write_fmt(format_args!("BigInt({:?})", array))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid decimal number")]
pub struct InvalidDecimalNumber;

impl BigInt {
    pub fn zero() -> Self {
        mina_curves::pasta::Fp::from(0u64).into()
    }

    pub fn one() -> Self {
        mina_curves::pasta::Fp::from(1u64).into()
    }

    pub fn to_field<F>(&self) -> Result<F, InvalidBigInt>
    where
        F: ark_ff::Field + TryFrom<BigInteger256, Error = InvalidBigInt>,
    {
        let Self(biginteger) = self;
        F::try_from(*biginteger)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        use ark_ff::ToBytes;
        let mut bytes = std::io::Cursor::new([0u8; 32]);
        self.0.write(&mut bytes).unwrap(); // Never fail, there is 32 bytes
        bytes.into_inner()
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        use ark_ff::FromBytes;
        Self(BigInteger256::read(&bytes[..]).unwrap()) // Never fail, we read from 32 bytes
    }

    pub fn from_decimal(s: &str) -> Result<Self, InvalidDecimalNumber> {
        num_bigint::BigInt::parse_bytes(s.as_bytes(), 10)
            .map(|num| {
                let mut bytes = num.to_bytes_be().1;
                bytes.reverse();
                bytes.resize(32, 0); // Ensure the byte vector has 32 bytes
                BigInt::from_bytes(bytes.try_into().unwrap())
            })
            .ok_or(InvalidDecimalNumber)
    }

    pub fn to_decimal(&self) -> String {
        let bigint: num_bigint::BigUint = self.0.into();
        bigint.to_string()
    }
}

impl AsRef<BigInteger256> for BigInt {
    fn as_ref(&self) -> &BigInteger256 {
        let Self(biginteger) = self;
        biginteger
    }
}

impl From<mina_curves::pasta::Fp> for BigInt {
    fn from(field: mina_curves::pasta::Fp) -> Self {
        use ark_ff::PrimeField;
        Self(field.into_repr())
    }
}

impl From<mina_curves::pasta::Fq> for BigInt {
    fn from(field: mina_curves::pasta::Fq) -> Self {
        use ark_ff::PrimeField;
        Self(field.into_repr())
    }
}

impl From<&mina_curves::pasta::Fp> for BigInt {
    fn from(field: &mina_curves::pasta::Fp) -> Self {
        use ark_ff::PrimeField;
        Self(field.into_repr())
    }
}

impl From<&mina_curves::pasta::Fq> for BigInt {
    fn from(field: &mina_curves::pasta::Fq) -> Self {
        use ark_ff::PrimeField;
        Self(field.into_repr())
    }
}

impl TryFrom<BigInt> for mina_curves::pasta::Fp {
    type Error = <mina_curves::pasta::Fp as TryFrom<BigInteger256>>::Error;
    fn try_from(bigint: BigInt) -> Result<Self, Self::Error> {
        bigint.to_field()
    }
}

impl TryFrom<BigInt> for mina_curves::pasta::Fq {
    type Error = <mina_curves::pasta::Fq as TryFrom<BigInteger256>>::Error;
    fn try_from(bigint: BigInt) -> Result<Self, Self::Error> {
        bigint.to_field()
    }
}

impl TryFrom<&BigInt> for mina_curves::pasta::Fp {
    type Error = <mina_curves::pasta::Fp as TryFrom<BigInteger256>>::Error;
    fn try_from(bigint: &BigInt) -> Result<Self, Self::Error> {
        bigint.to_field()
    }
}

impl TryFrom<&BigInt> for mina_curves::pasta::Fq {
    type Error = <mina_curves::pasta::Fq as TryFrom<BigInteger256>>::Error;
    fn try_from(bigint: &BigInt) -> Result<Self, Self::Error> {
        bigint.to_field()
    }
}

impl OfSexp for BigInt {
    fn of_sexp(s: &rsexp::Sexp) -> Result<Self, rsexp::IntoSexpError>
    where
        Self: Sized,
    {
        let bytes = s.extract_atom("BigInt")?;
        let hex_str = std::str::from_utf8(bytes).map_err(|_| {
            rsexp::IntoSexpError::StringConversionError {
                err: format!("Expected hex string with 0x prefix, got {bytes:?}"),
            }
        })?;

        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);

        let padded_hex = format!("{:0>64}", hex_str);

        if padded_hex.len() != 64 {
            return Err(rsexp::IntoSexpError::StringConversionError {
                err: format!("Expected 64-character hex string, got {padded_hex:?}"),
            });
        }

        let byte_vec: Vec<u8> = (0..padded_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&padded_hex[i..i + 2], 16))
            .rev()
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| rsexp::IntoSexpError::StringConversionError {
                err: format!("Failed to parse hex string: {padded_hex:?}"),
            })?;

        Ok(BigInt::from_bytes(byte_vec.try_into().unwrap()))
    }
}

impl SexpOf for BigInt {
    fn sexp_of(&self) -> rsexp::Sexp {
        use std::fmt::Write;
        let byte_vec = self.to_bytes();
        let hex_str = byte_vec
            .iter()
            .rev()
            .fold("0x".to_string(), |mut output, byte| {
                let _ = write!(output, "{byte:02X}");
                output
            });

        rsexp::Sexp::Atom(hex_str.into_bytes())
    }
}

impl binprot::BinProtRead for BigInt {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        use ark_ff::FromBytes;
        Ok(Self(BigInteger256::read(r)?))
    }
}

impl binprot::BinProtWrite for BigInt {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        use ark_ff::ToBytes;
        let Self(biginteger) = self;
        biginteger.write(w)
    }
}

impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            // TODO get rid of copying
            let mut rev = self.to_bytes();
            rev[..].reverse();
            let mut hex = [0_u8; 32 * 2 + 2];
            hex[..2].copy_from_slice(b"0x");
            hex::encode_to_slice(rev, &mut hex[2..]).unwrap();
            serializer.serialize_str(String::from_utf8_lossy(&hex).as_ref())
        } else {
            serializer.serialize_bytes(&self.to_bytes())
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
                    formatter.write_str("hex string or numeric string")
                }

                fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match v.strip_prefix("0x") {
                        Some(v) => hex::decode(v).map_err(|_| {
                            serde::de::Error::custom(format!("failed to decode hex str: {v}"))
                        }),
                        None => {
                            // Try to parse as a decimal number
                            num_bigint::BigInt::parse_bytes(v.as_bytes(), 10)
                                .map(|num| {
                                    let mut bytes = num.to_bytes_be().1;
                                    bytes.reverse();
                                    bytes.resize(32, 0); // Ensure the byte vector has 32 bytes
                                    bytes.reverse();
                                    bytes
                                })
                                .ok_or_else(|| {
                                    serde::de::Error::custom(
                                        "failed to parse decimal number".to_string(),
                                    )
                                })
                        }
                    }
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_borrowed_str(v)
                }
            }
            let mut v = deserializer.deserialize_str(V)?;
            v.reverse();
            v.try_into()
                .map_err(|_| serde::de::Error::custom("failed to convert vec to array".to_string()))
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
        .map(Self::from_bytes)
    }
}

impl mina_hasher::Hashable for BigInt {
    type D = ();

    fn to_roinput(&self) -> mina_hasher::ROInput {
        mina_hasher::ROInput::new()
            .append_field(self.to_field().expect("Failed to convert Hash into Fp"))
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
        BigInt::from_bytes([b; 32])
    }

    fn from_bytes<'a, I>(it: I) -> BigInt
    where
        I: IntoIterator<Item = &'a u8>,
        I::IntoIter: Clone,
    {
        let mut bytes = [0; 32];
        let it = it.into_iter().cycle();
        bytes.iter_mut().zip(it).for_each(|(b, i)| *b = *i);
        BigInt::from_bytes(bytes)
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
            assert_eq!(binprot.as_slice(), bigint.to_bytes());
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
            let deser: BigInt = from_binprot(&bigint.to_bytes());
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
            let mut v = bigint.to_bytes();
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
            let mut be = bigint.to_bytes();
            be.reverse();
            let json = format!(r#""0x{}""#, hex::encode(be.as_ref()));
            let bigint_exp = serde_json::from_str(&json).unwrap();
            assert_eq!(bigint, bigint_exp);
        }
    }

    #[test]
    fn from_numeric_string() {
        let hex = "075bcd1500000000000000000000000000000000000000000000000000000000";
        let deser: BigInt = serde_json::from_str(r#""123456789""#).unwrap();

        let mut deser = deser.to_bytes();
        deser.reverse();
        let result_hex = hex::encode(deser);

        assert_eq!(result_hex, hex.to_string());
    }

    #[test]
    fn from_numeric_string_2() {
        let rx =
            r#""23298604903871047876308234794524469025218548053411207476198573374353464993732""#;
        let s = r#""160863098041039391219472069845715442980741444645399750596310972807022542440""#;

        let deser_rx: BigInt = serde_json::from_str(rx).unwrap();
        let deser_s: BigInt = serde_json::from_str(s).unwrap();

        println!("rx: {:?}", deser_rx);
        println!("s: {:?}", deser_s);

        let _ = deser_rx.to_field::<mina_hasher::Fp>().unwrap();
        println!("rx OK");
        let _ = deser_s.to_field::<mina_hasher::Fp>().unwrap();
        println!("s OK");
    }

    use super::*;
    use rsexp::Sexp;

    #[test]
    fn test_sexp_bigint() {
        let hex_str = "0x248D179F4E92EA85C644CD99EF72187463B541D5F797943898C3D7A6CEEEC523";
        let expected_array = [
            0x98C3D7A6CEEEC523,
            0x63B541D5F7979438,
            0xC644CD99EF721874,
            0x248D179F4E92EA85,
        ];

        let original_sexp = Sexp::Atom(hex_str.as_bytes().to_vec());

        let result = BigInt::of_sexp(&original_sexp).expect("Failed to convert Sexp to BigInt");
        let expected_result = BigInt(BigInteger256::new(expected_array));

        assert_eq!(result, expected_result);

        let produced_sexp = result.sexp_of();

        assert_eq!(original_sexp, produced_sexp);
    }
}

use std::{fmt::Display, marker::PhantomData, str::FromStr};

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(
    Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::From, derive_more::Deref,
)]
pub struct Number<T>(pub T);

impl<T: std::fmt::Debug> std::fmt::Debug for Number<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Avoid vertical alignment
        f.write_fmt(format_args!("Number({inner:?})", inner = self.0))
    }
}

pub type Int32 = Number<i32>;
pub type UInt32 = Number<u32>;
pub type Int64 = Number<i64>;
pub type UInt64 = Number<u64>;
pub type Float64 = Number<f64>;

impl Int32 {
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
}

impl Int64 {
    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }
}

impl UInt32 {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl UInt64 {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u32> for Number<i32> {
    fn from(value: u32) -> Self {
        Self(value as i32)
    }
}

impl From<u64> for Number<i64> {
    fn from(value: u64) -> Self {
        Self(value as i64)
    }
}

impl From<&u32> for Number<i32> {
    fn from(value: &u32) -> Self {
        Self(*value as i32)
    }
}

impl From<&u64> for Number<i64> {
    fn from(value: &u64) -> Self {
        Self(*value as i64)
    }
}

impl From<&u32> for Number<u32> {
    fn from(value: &u32) -> Self {
        Self(*value)
    }
}

impl From<&u64> for Number<u64> {
    fn from(value: &u64) -> Self {
        Self(*value)
    }
}

impl<T> Serialize for Number<T>
where
    T: Serialize + Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !serializer.is_human_readable() {
            return self.0.serialize(serializer);
        }
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de, T> Deserialize<'de> for Number<T>
where
    T: Deserialize<'de> + FromStr,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !deserializer.is_human_readable() {
            return T::deserialize(deserializer).map(Self);
        }
        struct V<T>(PhantomData<T>);
        impl<'de, T> Visitor<'de> for V<T>
        where
            T: FromStr,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("stringified number")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|_| {
                    serde::de::Error::custom(format!("failed to parse string as number"))
                })
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|_| {
                    serde::de::Error::custom(format!("failed to parse string as number"))
                })
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|_| {
                    serde::de::Error::custom(format!("failed to parse string as number"))
                })
            }
        }
        deserializer
            .deserialize_string(V::<T>(Default::default()))
            .map(Self)
    }
}

macro_rules! binprot_number {
    ($base_type:ident, $binprot_type:ident) => {
        impl binprot::BinProtRead for Number<$base_type> {
            fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
            where
                Self: Sized,
            {
                $binprot_type::binprot_read(r).map(|v| Self(v as $base_type))
            }
        }

        impl binprot::BinProtWrite for Number<$base_type> {
            fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
                (self.0 as $binprot_type).binprot_write(w)
            }
        }
    };
}

binprot_number!(i32, i32);
binprot_number!(i64, i64);
binprot_number!(u32, i32);
binprot_number!(u64, i64);
binprot_number!(f64, f64);

#[cfg(test)]
mod tests {
    use binprot::{BinProtRead, BinProtWrite};

    macro_rules! number_test {
        ($name:ident, $ty:ident) => {
            #[test]
            fn $name() {
                for n in [
                    0,
                    1,
                    u8::MAX as $ty,
                    u16::MAX as $ty,
                    u32::MAX as $ty,
                    u64::MAX as $ty,
                    i8::MAX as $ty,
                    i16::MAX as $ty,
                    i32::MAX as $ty,
                    i64::MAX as $ty,
                ] {
                    let n: super::Number<$ty> = n.into();
                    let mut buf = Vec::new();
                    n.binprot_write(&mut buf).unwrap();
                    let mut r = buf.as_slice();
                    let n_ = super::Number::<$ty>::binprot_read(&mut r).unwrap();
                    assert_eq!(r.len(), 0);
                    assert_eq!(n, n_);
                }
            }
        };
    }

    macro_rules! max_number_test {
        ($name:ident, $ty:ident) => {
            #[test]
            fn $name() {
                let binprot = b"\xff\xff";
                let mut r = &binprot[..];
                let n = super::Number::<$ty>::binprot_read(&mut r).unwrap();
                assert_eq!(n.0, $ty::MAX);

                let n: super::Number<$ty> = $ty::MAX.into();
                let mut buf = Vec::new();
                n.binprot_write(&mut buf).unwrap();
                assert_eq!(buf.as_slice(), b"\xff\xff");
            }
        };
    }

    number_test!(i32_roundtrip, i32);
    number_test!(u32_roundtrip, u32);
    number_test!(i64_roundtrip, i64);
    number_test!(u64_roundtrip, u64);

    max_number_test!(u32_max, u32);
    max_number_test!(u64_max, u64);
}

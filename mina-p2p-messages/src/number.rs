use std::{fmt::Display, marker::PhantomData, str::FromStr};

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(
    Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::From, derive_more::Deref,
)]
pub struct Number<T>(pub T);

impl<T: std::fmt::Debug> std::fmt::Debug for Number<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        // Avoid vertical alignment
        f.write_fmt(format_args!("Number({:?})", inner))
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

impl<T> binprot::BinProtRead for Number<T>
where
    T: binprot::BinProtRead,
{
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        T::binprot_read(r).map(Self)
    }
}

impl<T> binprot::BinProtWrite for Number<T>
where
    T: binprot::BinProtWrite,
{
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.0.binprot_write(w)
    }
}

#[cfg(test)]
mod tests {
    use binprot::{BinProtRead, BinProtWrite};

    #[test]
    fn u32_roundtrip() {
        for u32 in [
            0,
            1,
            u8::MAX as u32,
            u16::MAX as u32,
            u32::MAX,
            i8::MAX as u32,
            i16::MAX as u32,
            i32::MAX as u32,
        ] {
            let mut buf = Vec::new();
            u32.binprot_write(&mut buf).unwrap();
            let mut r = buf.as_slice();
            if u32 <= 0x7f {
                assert_eq!(r[0], u32 as u8);
            } else {
                assert!(matches!(r[0], 0xfe | 0xfd | 0xfc));
            }
            let u32_ = u32::binprot_read(&mut r).unwrap();
            assert_eq!(r.len(), 0);
            assert_eq!(u32, u32_);
        }
    }

    #[test]
    fn i32_roundtrip() {
        for i32 in [
            0,
            1,
            u8::MAX as i32,
            u16::MAX as i32,
            u32::MAX as i32,
            i8::MAX as i32,
            i16::MAX as i32,
            i32::MAX as i32,
            i8::MIN as i32,
            i16::MIN as i32,
            i32::MIN as i32,
        ] {
            let mut buf = Vec::new();
            i32.binprot_write(&mut buf).unwrap();
            let mut r = buf.as_slice();
            if -0x80 <= i32 && i32 < 0 {
                assert_eq!(r[0], 0xff);
            } else if 0 <= i32 && i32 <= 0x80 {
                assert_eq!(r[0], i32 as u8);
            } else {
                assert!(matches!(r[0], 0xfe | 0xfd | 0xfc));
            }
            let i32_ = i32::binprot_read(&mut r).unwrap();
            assert_eq!(r.len(), 0);
            assert_eq!(i32, i32_);
        }
    }
}

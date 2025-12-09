use std::{fmt::Display, marker::PhantomData, str::FromStr};

use malloc_size_of::MallocSizeOf;
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

impl<T> MallocSizeOf for Number<T> {
    fn size_of(&self, _ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        0
    }
}

pub type Int32 = Number<i32>;
pub type UInt32 = Number<u32>;
pub type Int64 = Number<i64>;
pub type UInt64 = Number<u64>;
pub type Float64 = Number<f64>;

impl Int32 {
    pub const fn as_u32(&self) -> u32 {
        self.0 as u32
    }
}

impl Int64 {
    pub const fn as_u64(&self) -> u64 {
        self.0 as u64
    }
}

impl UInt32 {
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl UInt64 {
    pub const fn as_u64(&self) -> u64 {
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
                formatter.write_str("a stringified number or a literal integer")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|_| {
                    serde::de::Error::custom("failed to parse string as number".to_string())
                })
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|_| {
                    serde::de::Error::custom("failed to parse string as number".to_string())
                })
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|_| {
                    serde::de::Error::custom("failed to parse string as number".to_string())
                })
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let s = v.to_string();
                s.parse()
                    .map_err(|_| serde::de::Error::custom("failed to parse integer as number"))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let s = v.to_string();
                s.parse().map_err(|_| {
                    serde::de::Error::custom("failed to parse unsigned integer as number")
                })
            }
        }
        deserializer
            .deserialize_any(V::<T>(Default::default()))
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
binprot_number!(f64, f64);

// Custom implementations for u32 and u64 to handle the full i64 range
// without TryFromIntError. The binprot format stores all integers as i64
// internally, and conversion can fail if the value is outside the target
// type's range. We read as i64 first, then cast to handle out-of-range values
// gracefully. Writing still uses the smaller type to maintain wire format
// compatibility.
impl binprot::BinProtRead for Number<u32> {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        // Read the value as i64 first to avoid TryFromIntError when the value
        // is outside i32 range, then cast to u32 using wrapping semantics.
        let value = i64::binprot_read(r)?;
        Ok(Self(value as u32))
    }
}

impl binprot::BinProtWrite for Number<u32> {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        // Write as i32 to maintain wire format compatibility
        (self.0 as i32).binprot_write(w)
    }
}

impl binprot::BinProtRead for Number<u64> {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        // Read the value as i64 first to handle the full range without errors,
        // then cast to u64 using wrapping semantics.
        let value = i64::binprot_read(r)?;
        Ok(Self(value as u64))
    }
}

impl binprot::BinProtWrite for Number<u64> {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        // Write as i64 to maintain wire format compatibility
        (self.0 as i64).binprot_write(w)
    }
}

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

    /// Test that Number<u32> can read i64 values outside i32 range without error.
    /// This is a regression test for the TryFromIntError issue.
    #[test]
    fn u32_read_large_i64() {
        // Create a binprot-encoded i64 value larger than i32::MAX
        let large_i64 = (i32::MAX as i64) + 1000;
        let mut buf = Vec::new();
        large_i64.binprot_write(&mut buf).unwrap();

        // This should not fail with TryFromIntError
        let mut r = buf.as_slice();
        let result = super::Number::<u32>::binprot_read(&mut r);
        assert!(result.is_ok(), "Should handle i64 values outside i32 range");

        // The value should wrap as expected when cast from i64 to u32
        let n = result.unwrap();
        assert_eq!(n.0, large_i64 as u32);
    }

    /// Test that Number<u32> can read negative i64 values without error.
    #[test]
    fn u32_read_negative_i64() {
        // Create a binprot-encoded negative i64 value
        let negative_i64 = -1000i64;
        let mut buf = Vec::new();
        negative_i64.binprot_write(&mut buf).unwrap();

        // This should not fail with TryFromIntError
        let mut r = buf.as_slice();
        let result = super::Number::<u32>::binprot_read(&mut r);
        assert!(result.is_ok(), "Should handle negative i64 values");

        // The value should wrap as expected when cast from i64 to u32
        let n = result.unwrap();
        assert_eq!(n.0, negative_i64 as u32);
    }

    /// Test that Number<u64> can read i64 values without error.
    #[test]
    fn u64_read_i64() {
        // Test with a negative value (which would fail with try_from)
        let negative_i64 = -5000i64;
        let mut buf = Vec::new();
        negative_i64.binprot_write(&mut buf).unwrap();

        let mut r = buf.as_slice();
        let result = super::Number::<u64>::binprot_read(&mut r);
        assert!(result.is_ok(), "Should handle negative i64 values");

        let n = result.unwrap();
        assert_eq!(n.0, negative_i64 as u64);
    }
}

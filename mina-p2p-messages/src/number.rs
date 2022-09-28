use std::{fmt::Display, marker::PhantomData, str::FromStr};

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, derive_more::From)]
pub struct Number<T>(T);

pub type Int32 = Number<i32>;
pub type Int64 = Number<i64>;
pub type Float64 = Number<f64>;

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
                E: serde::de::Error, {
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

use std::{fmt::Debug, marker::PhantomData};

use serde::{ser::SerializeStruct, Deserialize, Serialize};

/// `Bin_prot` uses integer to represent type version.
pub type Ver = i32;

/// Wrapper for a type that adds explicit version information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Versioned<T, const V: Ver>(T);

impl<T, const V: Ver> From<T> for Versioned<T, V> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

impl<T, const V: Ver> Serialize for Versioned<T, V>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.0.serialize(serializer)
        } else {
            let mut s = serializer.serialize_struct("MakeVersioned", 2)?;
            s.serialize_field("version", &V)?;
            s.serialize_field("t", &self.0)?;
            s.end()
        }
    }
}

impl<'de, T, const V: Ver> Deserialize<'de> for Versioned<T, V>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FieldsVisitor<T, const V: Ver>(PhantomData<T>);
        impl<'de, T, const V: Ver> serde::de::Visitor<'de> for FieldsVisitor<T, V>
        where
            T: Deserialize<'de>,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expecting a struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let version: Ver = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                if version != V {
                    return Err(serde::de::Error::custom(format!(
                        "invalid version, expecting {}, actual {version}",
                        V
                    )));
                }
                let t = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                Ok(t)
            }
        }

        const FIELDS: &'static [&'static str] = &["version", "t"];
        deserializer
            .deserialize_struct(
                "MakeVersioned",
                FIELDS,
                FieldsVisitor::<T, V>(Default::default()),
            )
            .map(Self)
    }
}

#[derive(Debug)]
pub struct VersionMismatchError {
    expected: Ver,
    actual: Ver,
}

impl std::fmt::Display for VersionMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "version mismatch, expected {}, actual {}", self.expected, self.actual)
    }
}

impl std::error::Error for VersionMismatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl<T, const V: Ver> binprot::BinProtRead for Versioned<T, V>
where
    T: binprot::BinProtRead,
{
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let version: Ver = binprot::BinProtRead::binprot_read(r)?;
        if version != V {
            return Err(binprot::Error::CustomError(Box::new(
                VersionMismatchError {
                    expected: V,
                    actual: version,
                },
            )));
        }
        let t: T = binprot::BinProtRead::binprot_read(r)?;
        Ok(Self(t))
    }
}

impl<T, const V: Ver> binprot::BinProtWrite for Versioned<T, V>
where
    T: binprot::BinProtWrite,
{
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        binprot::BinProtWrite::binprot_write(&V, w)?;
        binprot::BinProtWrite::binprot_write(&self.0, w)
    }
}



/*
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::versioned::{Ver, VersionTrait, Versioned, WithVersion};


    #[test]
    fn serde_versioned() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Foo {
            a: u8,
            b: u32,
        }

        let foo = Foo { a: 0x7f, b: 0xffff };
        let foo_bin_prot = b"\x7f\xfe\xff\xff";
        let foo_v_bin_prot = b"\x01\x7f\xfe\xff\xff";
        let foo_json = serde_json::json!({"a": 0x7f, "b": 0xffff});

        let bytes = serde_binprot::to_vec(&foo).unwrap();
        assert_eq!(&bytes, foo_bin_prot);

        let json = serde_json::to_value(&foo).unwrap();
        assert_eq!(json, foo_json);

        let foo_de: Foo = serde_binprot::from_slice(foo_bin_prot).unwrap();
        assert_eq!(foo_de, foo);

        impl VersionTrait for Foo {
            const VERSION: Ver = 1;
        }
        let foo = Versioned::from(foo);

        let bytes = serde_binprot::to_vec(&foo).unwrap();
        assert_eq!(&bytes, foo_v_bin_prot);

        let json = serde_json::to_value(&foo).unwrap();
        assert_eq!(json, foo_json);

        let foo_de: Versioned<Foo> = serde_binprot::from_slice(foo_v_bin_prot).unwrap();
        assert_eq!(foo_de.0, foo.0);
    }

    #[test]
    fn serde_with_version() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Foo {
            a: u8,
            b: u32,
        }

        let foo = Foo { a: 0x7f, b: 0xffff };
        let foo_bin_prot = b"\x7f\xfe\xff\xff";
        let foo_v_bin_prot = b"\x02\x7f\xfe\xff\xff";
        let foo_json = serde_json::json!({"a": 0x7f, "b": 0xffff});

        let bytes = serde_binprot::to_vec(&foo).unwrap();
        assert_eq!(&bytes, foo_bin_prot);

        let json = serde_json::to_value(&foo).unwrap();
        assert_eq!(json, foo_json);

        let foo_de: Foo = serde_binprot::from_slice(foo_bin_prot).unwrap();
        assert_eq!(foo_de, foo);

        let foo: WithVersion<Foo, 2> = WithVersion(foo);

        let bytes = serde_binprot::to_vec(&foo).unwrap();
        assert_eq!(&bytes, foo_v_bin_prot);

        let json = serde_json::to_value(&foo).unwrap();
        assert_eq!(json, foo_json);

        let foo_de: WithVersion<Foo, 2> = serde_binprot::from_slice(foo_v_bin_prot).unwrap();
        assert_eq!(foo_de.0, foo.0);
    }
}
*/

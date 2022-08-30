use std::{fmt::Debug, marker::PhantomData};

use serde::{ser::SerializeStruct, Deserialize, Serialize};

/// `Bin_prot` uses integer to represent type version.
pub type Ver = i32;

/// Wrapper for a type that adds explicit version information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Versioned<T, const V: Ver>(T);

impl<T, const V: Ver> Versioned<T, V> {
    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

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
            return self.0.serialize(serializer);
        }
        let mut s = serializer.serialize_struct("MakeVersioned", 2)?;
        s.serialize_field("version", &V)?;
        s.serialize_field("t", &self.0)?;
        s.end()
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
        if deserializer.is_human_readable() {
            return T::deserialize(deserializer).map(Self);
        }
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
        write!(
            f,
            "version mismatch, expected {}, actual {}",
            self.expected, self.actual
        )
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

#[cfg(test)]
mod tests {
    use binprot::{BinProtRead, BinProtWrite};
    use binprot_derive::{BinProtRead, BinProtWrite};
    use serde::{Deserialize, Serialize};

    use crate::versioned::Versioned;

    fn binprot_read<T>(buf: &[u8]) -> Result<(T, &[u8]), binprot::Error>
    where
        T: BinProtRead,
    {
        let mut rest = buf;
        let res = T::binprot_read(&mut rest)?;
        Ok((res, rest))
    }

    fn binprot_write<T>(t: &T) -> std::io::Result<Vec<u8>>
    where
        T: BinProtWrite,
    {
        let mut buf = Vec::new();
        let _ = t.binprot_write(&mut buf)?;
        Ok(buf)
    }

    #[test]
    fn binprot() {
        #[derive(Debug, Serialize, Deserialize, PartialEq, BinProtRead, BinProtWrite)]
        struct Foo {
            a: u8,
            b: u32,
        }

        for (foo, foo_bin_prot) in [
            (Foo { a: 0x00, b: 0x00 }, b"\x00\x00" as &[u8]),
            (Foo { a: 0x01, b: 0x01 }, b"\x01\x01"),
            (Foo { a: 0x7f, b: 0x7fff }, b"\x7f\xfe\xff\x7f"),
        ] {
            let foo_json = serde_json::json!({"a": foo.a, "b": foo.b});

            let bytes = binprot_write(&foo).unwrap();
            assert_eq!(&bytes, foo_bin_prot);

            let json = serde_json::to_value(&foo).unwrap();
            assert_eq!(json, foo_json);

            let (foo_de, rest) = binprot_read::<Foo>(foo_bin_prot).unwrap();
            assert_eq!(rest.len(), 0);
            assert_eq!(&foo_de, &foo);
        }
    }

    #[test]
    fn binprot_versioned() {
        #[derive(Debug, Serialize, Deserialize, PartialEq, BinProtRead, BinProtWrite)]
        struct Foo {
            a: u8,
            b: u32,
        }

        for (foo, foo_bin_prot) in [
            (Foo { a: 0x00, b: 0x00 }, b"\x01\x00\x00" as &[u8]),
            (Foo { a: 0x01, b: 0x01 }, b"\x01\x01\x01"),
            (Foo { a: 0x7f, b: 0x7fff }, b"\x01\x7f\xfe\xff\x7f"),
        ] {
            type VersionedFoo = Versioned<Foo, 1>;
            let foo_json = serde_json::json!({"a": foo.a, "b": foo.b});
            let foo = Versioned::from(foo);

            let bytes = binprot_write(&foo).unwrap();
            assert_eq!(&bytes, foo_bin_prot);

            let json = serde_json::to_value(&foo).unwrap();
            assert_eq!(json, foo_json);

            let (foo_de, rest) = binprot_read::<VersionedFoo>(foo_bin_prot).unwrap();
            assert_eq!(rest.len(), 0);
            assert_eq!(&foo_de, &foo);
        }
    }

    #[test]
    fn binprot_version_num_write() {
        fn versioned<const V: i32>() -> Versioned<(), V> {
            Versioned(())
        }
        assert_eq!(&binprot_write(&versioned::<0>()).unwrap(), b"\x00\x00");
        assert_eq!(&binprot_write(&versioned::<1>()).unwrap(), b"\x01\x00");
        assert_eq!(&binprot_write(&versioned::<0x7f>()).unwrap(), b"\x7f\x00");
        assert_eq!(
            &binprot_write(&versioned::<0x80>()).unwrap(),
            b"\xfe\x80\x00\x00"
        );
    }
}

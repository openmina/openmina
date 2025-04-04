use std::{array, fmt::Formatter, marker::PhantomData};

use binprot::{BinProtRead, BinProtWrite};
use malloc_size_of_derive::MallocSizeOf;
use rsexp::{OfSexp, SexpOf};
use serde::ser::SerializeTuple;
#[derive(Clone, Debug, PartialEq, MallocSizeOf)]
pub struct PaddedSeq<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> Default for PaddedSeq<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self(array::from_fn(|_| T::default()))
    }
}

impl<T, const N: usize> std::ops::Deref for PaddedSeq<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: OfSexp, const N: usize> OfSexp for PaddedSeq<T, N> {
    fn of_sexp(s: &rsexp::Sexp) -> Result<Self, rsexp::IntoSexpError>
    where
        Self: Sized,
    {
        let elts = s.extract_list("PaddedSeq")?;
        if elts.len() != N {
            return Err(rsexp::IntoSexpError::ListLengthMismatch {
                type_: "PaddedSeq",
                expected_len: N,
                list_len: elts.len(),
            });
        }

        let mut converted: [Option<T>; N] = [(); N].map(|_| None);

        for (i, item) in elts.iter().enumerate() {
            converted[i] = Some(T::of_sexp(item)?);
        }

        // Unwrap cannot fail, otherwise we wouldn't have rechead this point
        Ok(Self(converted.map(|item| item.unwrap())))
    }
}

impl<T: SexpOf, const N: usize> rsexp::SexpOf for PaddedSeq<T, N> {
    fn sexp_of(&self) -> rsexp::Sexp {
        let elements: Vec<rsexp::Sexp> = self.0.iter().map(|item| item.sexp_of()).collect();

        rsexp::Sexp::List(elements)
    }
}

impl<T: BinProtRead, const N: usize> binprot::BinProtRead for PaddedSeq<T, N> {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let mut vec = Vec::with_capacity(N);
        for _i in 0..N {
            vec.push(BinProtRead::binprot_read(r)?);
        }
        let _: () = BinProtRead::binprot_read(r)?;
        match vec.try_into() {
            Ok(arr) => Ok(PaddedSeq(arr)),
            Err(_) => unreachable!(),
        }
    }
}

impl<T: BinProtWrite, const N: usize> binprot::BinProtWrite for PaddedSeq<T, N> {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        for elt in &self.0 {
            elt.binprot_write(w)?;
        }
        ().binprot_write(w)?;
        Ok(())
    }
}

impl<T, const N: usize> serde::Serialize for PaddedSeq<T, N>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serializer = serializer.serialize_tuple(N + 1)?;
        for elt in &self.0 {
            serializer.serialize_element(elt)?;
        }
        serializer.end()
    }
}

impl<'de, T, const N: usize> serde::Deserialize<'de> for PaddedSeq<T, N>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'de, T, const S: usize>
        where
            T: serde::Deserialize<'de>,
        {
            marker: PhantomData<PaddedSeq<T, S>>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de, T, const S: usize> serde::de::Visitor<'de> for Visitor<'de, T, S>
        where
            T: serde::Deserialize<'de>,
        {
            type Value = PaddedSeq<T, S>;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                Formatter::write_str(formatter, "tuple struct PaddedSeq")
            }
            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::with_capacity(S);
                for i in 0..S {
                    match serde::de::SeqAccess::next_element(&mut seq)? {
                        Some(value) => vec.push(value),
                        None => {
                            return Err(serde::de::Error::invalid_length(
                                i,
                                &concat!(
                                    "tuple struct PaddedSeq with ",
                                    stringify!(S),
                                    " element(s)"
                                ),
                            ));
                        }
                    }
                }
                let res = match <[T; S]>::try_from(vec) {
                    Ok(a) => a,
                    Err(_) => unreachable!(),
                };
                Ok(PaddedSeq(res))
            }
        }
        deserializer.deserialize_tuple(
            N,
            Visitor {
                marker: PhantomData::<PaddedSeq<T, N>>,
                lifetime: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_json() {
        let v = PaddedSeq([1, 2, 3]);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(&json, "[1,2,3]");
    }

    #[test]
    fn from_json() {
        let json = "[1, 2, 3]";
        let v = serde_json::from_str::<PaddedSeq<_, 3>>(json).unwrap();
        assert_eq!(v, PaddedSeq([1, 2, 3]));
    }

    #[test]
    fn to_binprot() {
        let v = PaddedSeq([1, 2, 3]);
        let mut binprot = Vec::new();
        v.binprot_write(&mut binprot).unwrap();
        assert_eq!(&binprot, b"\x01\x02\x03\x00");
    }

    #[test]
    fn from_binprot() {
        let binprot = b"\x01\x02\x03\x00";
        let v = PaddedSeq::<_, 3>::binprot_read(&mut &binprot[..]).unwrap();
        assert_eq!(v, PaddedSeq([1, 2, 3]));
    }
}

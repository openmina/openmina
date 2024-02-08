use std::ops::{Deref, DerefMut};

use binprot::{BinProtRead, BinProtWrite, Nat0};

/// Represents OCaml list type.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    derive_more::From,
    derive_more::Into,
)]
pub struct List<T>(Vec<T>);

impl<T> List<T> {
    pub fn new() -> Self {
        List(Vec::new())
    }

    pub fn iter(&self) -> <&Vec<T> as IntoIterator>::IntoIter {
        (self).into_iter()
    }

    pub fn insert(&mut self, index: usize, element: T) {
        self.0.insert(index, element)
    }
}

impl<T> Deref for List<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;

    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;

    type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        List(Vec::from_iter(iter))
    }
}

impl<T> BinProtRead for List<T>
where
    T: BinProtRead,
{
    fn binprot_read<R: std::io::prelude::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error> {
        let Nat0(len) = Nat0::binprot_read(r)?;
        let mut v: Vec<T> = Vec::new();
        for _i in 0..len {
            let item = T::binprot_read(r)?;
            v.push(item)
        }
        Ok(List(v))
    }
}

impl<T> BinProtWrite for List<T>
where
    T: BinProtWrite,
{
    fn binprot_write<W: std::io::prelude::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let len = self.0.len() as u64;
        Nat0(len).binprot_write(w)?;
        for v in self.0.iter() {
            v.binprot_write(w)?
        }
        Ok(())
    }
}

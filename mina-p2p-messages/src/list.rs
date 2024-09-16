use std::{
    collections::LinkedList,
    ops::{Deref, DerefMut},
};

use binprot::{BinProtRead, BinProtWrite, Nat0};
use rsexp::OfSexp;

pub type Backend<T> = LinkedList<T>;

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
pub struct List<T>(Backend<T>);

impl<T> List<T> {
    pub fn new() -> Self {
        List(Backend::new())
    }

    pub fn one(elt: T) -> Self {
        let mut l = List(Backend::new());
        l.push_front(elt);
        l
    }

    // pub fn iter(&self) -> <&Backend<T> as IntoIterator>::IntoIter {
    //     (self).into_iter()
    // }

    pub fn push_front(&mut self, element: T) {
        self.0.push_front(element)
    }
}

impl<T: OfSexp> OfSexp for List<T> {
    fn of_sexp(s: &rsexp::Sexp) -> Result<Self, rsexp::IntoSexpError>
    where
        Self: Sized,
    {
        let elts = s.extract_list("List")?;
        let mut backend = Backend::new();
        for elt in elts.iter() {
            backend.push_back(rsexp::OfSexp::of_sexp(elt)?);
        }
        Ok(Self(backend))
    }
}

impl<T: rsexp::SexpOf> rsexp::SexpOf for List<T> {
    fn sexp_of(&self) -> rsexp::Sexp {
        let elements: Vec<rsexp::Sexp> = self.0.iter().map(|item| item.sexp_of()).collect();

        rsexp::Sexp::List(elements)
    }
}

impl<T> Deref for List<T> {
    type Target = Backend<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;

    type IntoIter = <Backend<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;

    type IntoIter = <&'a Backend<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        List(Backend::from_iter(iter))
    }
}

impl<T> BinProtRead for List<T>
where
    T: BinProtRead,
{
    fn binprot_read<R: std::io::prelude::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error> {
        let Nat0(len) = Nat0::binprot_read(r)?;
        let mut v: Backend<T> = Backend::new();
        for _i in 0..len {
            let item = T::binprot_read(r)?;
            v.push_back(item)
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

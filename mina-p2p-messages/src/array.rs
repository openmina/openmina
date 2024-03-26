use std::ops::Deref;

use binprot::{BinProtRead, BinProtWrite, Nat0};
use serde::{Deserialize, Serialize};

/// Mina array bounded to specific length. Note that the length is only checked
/// when performing binprot operations.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    derive_more::From,
    derive_more::Into,
)]
pub struct ArrayN<T, const N: u64>(Vec<T>);

impl<T, const N: u64> AsRef<[T]> for ArrayN<T, N> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T, const N: u64> Deref for ArrayN<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T, const N: u64> IntoIterator for ArrayN<T, N> {
    type Item = T;

    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, const N: u64> IntoIterator for &'a ArrayN<T, N> {
    type Item = &'a T;

    type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).iter()
    }
}

impl<T, const N: u64> FromIterator<T> for ArrayN<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ArrayN::<_, N>(Vec::from_iter(iter))
    }
}

impl<T, const N: u64> ArrayN<T, N> {
    pub fn to_inner(self) -> Vec<T> {
        self.0
    }

    pub fn inner(&self) -> &Vec<T> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}

impl<T, const N: u64> BinProtRead for ArrayN<T, N>
where
    T: BinProtRead,
{
    fn binprot_read<R: std::io::prelude::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error> {
        let Nat0(len) = Nat0::binprot_read(r)?;
        if len > N {
            return Err(MinaArrayNTooLong::<N>::new(len).into());
        }
        let mut v: Vec<T> = Vec::with_capacity(len as usize);
        for _i in 0..len {
            let item = T::binprot_read(r)?;
            v.push(item)
        }
        Ok(ArrayN(v))
    }
}

impl<T, const N: u64> BinProtWrite for ArrayN<T, N>
where
    T: BinProtWrite,
{
    fn binprot_write<W: std::io::prelude::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let len = self.0.len() as u64;
        if len > N {
            return Err(MinaArrayNTooLong::<N>::new(len).into());
        }
        Nat0(len).binprot_write(w)?;
        for v in self.0.iter() {
            v.binprot_write(w)?
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("String length `{0}` is greater than maximum `{N}`")]
pub struct MinaArrayNTooLong<const N: u64>(u64);

impl<const N: u64> MinaArrayNTooLong<N> {
    fn new(actual: u64) -> Self {
        MinaArrayNTooLong(actual)
    }
}

impl<const N: u64> From<MinaArrayNTooLong<N>> for std::io::Error {
    fn from(value: MinaArrayNTooLong<N>) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, Box::new(value))
    }
}

impl<const N: u64> From<MinaArrayNTooLong<N>> for binprot::Error {
    fn from(value: MinaArrayNTooLong<N>) -> Self {
        binprot::Error::CustomError(Box::new(value))
    }
}

/// Mina array limited to 16 elements.
pub type ArrayN16<T> = ArrayN<T, 16>;

/// Mina array limited to 4000 elements.
pub type ArrayN4000<T> = ArrayN<T, 4000>;

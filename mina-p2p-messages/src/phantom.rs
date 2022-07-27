use std::marker::PhantomData;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Phantom<T>(PhantomData<T>);

impl<T> binprot::BinProtRead for Phantom<T> {
    fn binprot_read<R: std::io::Read + ?Sized>(_r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized {
        Ok(Self(PhantomData))
    }
}

impl<T> binprot::BinProtWrite for Phantom<T> {
    fn binprot_write<W: std::io::Write>(&self, _w: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

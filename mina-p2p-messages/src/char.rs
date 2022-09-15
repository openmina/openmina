use binprot::byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Char(u8);

impl From<u8> for Char {
    fn from(source: u8) -> Self {
        Self(source)
    }
}

impl From<Char> for u8 {
    fn from(source: Char) -> Self {
        source.0
    }
}

impl binprot::BinProtRead for Char {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        Ok(r.read_u8().map(Self)?)
    }
}

impl binprot::BinProtWrite for Char {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_u8(self.0)
    }
}

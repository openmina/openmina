use std::net::IpAddr;

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};
use crate::{
    string::{ByteString, CharString},
    versioned::Versioned,
};

///! Types from Janestreet's Core library.

/// This type corresponds to `Bounded_types.Wrapped_error` OCaml type, but the
/// structure is different. It only refrects the data that is passed over the
/// wire (stringified sexp representation of the error)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, derive_more::Display)]
#[serde(into = "SexpString", try_from = "SexpString")]
#[display(fmt = "{}", "String::from_utf8_lossy(&_0.to_bytes())")]
pub struct Info(rsexp::Sexp);

impl Info {
    pub fn from_str(msg: &str) -> Self {
        Info(rsexp::atom(msg.as_bytes()))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error parsing info sexp: {0:?}")]
pub struct InfoFromSexpError(rsexp::Error);

impl binprot::BinProtRead for Info {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error> {
        let sexp = ByteString::binprot_read(r)?;
        let parsed = rsexp::from_slice(&sexp)
            .map_err(|e| binprot::Error::CustomError(Box::new(InfoFromSexpError(e))))?;
        Ok(Info(parsed))
    }
}

impl binprot::BinProtWrite for Info {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        ByteString::from(self.0.to_bytes()).binprot_write(w)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SexpString(CharString);

impl From<Info> for SexpString {
    fn from(value: Info) -> Self {
        SexpString(CharString::from(value.0.to_bytes()))
    }
}

impl TryFrom<SexpString> for Info {
    type Error = InfoFromSexpError;

    fn try_from(value: SexpString) -> Result<Self, Self::Error> {
        let parsed = rsexp::from_slice(&value.0)
            .map_err(|e| InfoFromSexpError(e))?;
        Ok(Info(parsed))
    }
}

/// Represents error processing an RPC request.
pub type Error = Info;

// TODO
#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq)]
pub struct Time(f64);

pub type InetAddrV1Versioned = Versioned<InetAddrV1, 1>;

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, derive_more::From, derive_more::Into,
)]
pub struct InetAddrV1(IpAddr);

impl binprot::BinProtRead for InetAddrV1 {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let s = String::binprot_read(r)?;
        let ip_addr: IpAddr = s
            .parse()
            .map_err(|e| binprot::Error::CustomError(Box::new(e)))?;
        Ok(ip_addr.into())
    }
}

impl binprot::BinProtWrite for InetAddrV1 {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.0.to_string().binprot_write(w)
    }
}

#[cfg(test)]
mod test {
    use binprot::{BinProtWrite, BinProtRead};

    use super::Info;

    fn info_to_bytes(info: &Info) -> Vec<u8> {
        let mut bytes = Vec::new();
        info.binprot_write(&mut bytes).unwrap();
        bytes
    }

    fn bytes_to_info(mut bytes: &[u8]) -> Info {
        let info = Info::binprot_read(&mut bytes).unwrap();
        assert!(bytes.len() == 0);
        info
    }

    #[test]
    fn sexp_binprot() {
        use rsexp::*;

        let info = Info(atom(b"atom"));
        let bytes = info_to_bytes(&info);
        assert_eq!(&bytes, b"\x04atom");
        assert_eq!(info, bytes_to_info(&bytes));

        let info = Info(atom(b"atom atom"));
        let bytes = info_to_bytes(&info);
        assert_eq!(&bytes, b"\x0b\"atom atom\"");
        assert_eq!(info, bytes_to_info(&bytes));
    }
}

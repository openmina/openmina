use std::net::IpAddr;

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::versioned::Versioned;

///! Types from Janestreet's Core library.

/// Core.Error type.
///
/// TODO properly implement payloads.
#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub enum Info {
    CouldNotConstruct(super::string::String), // of Sexp.t
    String(super::string::String),            // of string
    Exn,                                      // of Binable_exn.V1.t
    Sexp,                                     // of Sexp.t
    TagSexp,       // of string * Sexp.t * Source_code_position.V1.t option
    TagT,          // of string * t
    TagArg,        // of string * Sexp.t * t
    OfList,        // of int option * t list
    WithBacktrace, // of t * string (* backtrace *)
}

pub type Error = Info;

// TODO
#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq)]
pub struct Time(f64);

pub type InetAddrV1Binable = Versioned<InetAddrV1, 1>;

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

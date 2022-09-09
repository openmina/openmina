use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

/// Core.Error type.
///
/// TODO properly implement payloads.
#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub enum Info {
    CouldNotConstruct(super::string::String), // of Sexp.t
    String(super::string::String),            // of string
    Exn,                 // of Binable_exn.V1.t
    Sexp,                // of Sexp.t
    TagSexp,            // of string * Sexp.t * Source_code_position.V1.t option
    TagT,               // of string * t
    TagArg,             // of string * Sexp.t * t
    OfList,             // of int option * t list
    WithBacktrace,      // of t * string (* backtrace *)
}

pub type Error = Info;

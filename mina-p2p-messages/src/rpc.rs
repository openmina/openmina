use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

pub type Tag = super::string::String;
pub type QueryID = i64;
pub type RpcResult<T> = Result<T, Error>;
pub type Sexp = (); // TODO

/// RPC error.
///
/// ```ocaml
/// module Rpc_error : sig
///   open Core_kernel
///
///   type t =
///     | Bin_io_exn        of Sexp.t
///     | Connection_closed
///     | Write_error       of Sexp.t
///     | Uncaught_exn      of Sexp.t
///     | Unimplemented_rpc of Rpc_tag.t * [`Version of int]
///     | Unknown_query_id  of Query_id.t
///   [@@deriving bin_io, sexp, compare]
///
///   include Comparable.S with type t := t
/// end
/// ```
#[allow(non_camel_case_types)]
pub enum Error {
    Bin_io_exn(Sexp),
    Connection_closed,
    Write_error(Sexp),
    Uncaught_exn(Sexp),
    Unimplemented_rpc(Tag, super::versioned::Ver),
    Unknown_query_id(QueryID),
}

/// RPC query.
///
/// ```ocaml
/// module Query = struct
///   type 'a needs_length =
///     { tag     : Rpc_tag.t
///     ; version : int
///     ; id      : Query_id.t
///     ; data    : 'a
///     }
///   [@@deriving bin_io, sexp_of]
///   type 'a t = 'a needs_length [@@deriving bin_read]
/// end
/// ```
pub struct Query<T> {
    pub tag: Tag,
    pub version: i32,
    pub id: QueryID,
    pub data: T,
}

/// RPC response.
///
/// ```ocaml
/// module Response = struct
///   type 'a needs_length =
///     { id   : Query_id.t
///     ; data : 'a Rpc_result.t
///     }
///   [@@deriving bin_io, sexp_of]
///   type 'a t = 'a needs_length [@@deriving bin_read]
/// end
/// ```
pub struct Response<T> {
    pub id: QueryID,
    pub data: RpcResult<T>,
}

/// RPC message.
///
/// ```ocaml
/// module Message = struct
///   type 'a needs_length =
///     | Heartbeat
///     | Query     of 'a Query.   needs_length
///     | Response  of 'a Response.needs_length
///   [@@deriving bin_io, sexp_of]
///   type 'a t = 'a needs_length [@@deriving bin_read, sexp_of]
///   type nat0_t = Nat0.t needs_length [@@deriving bin_read, bin_write]
/// end
/// ```
pub enum Message<T> {
    Heartbeat,
    Query(Query<T>),
    Response(Response<T>),
}

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub struct QueryHeader {
    tag: Tag,
    version: i32,
    id: QueryID,
}

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub struct ResponseHeader {
    id: QueryID,
}

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub enum MessageHeader {
    Heartbeat,
    Query(QueryHeader),
    Response(ResponseHeader),
}

#[cfg(test)]
mod tests {
    use crate::utils::FromBinProtStream;

    use super::{MessageHeader, QueryHeader, ResponseHeader};

    #[test]
    fn message_header() {
        for (s, m) in [
            (
                "1e0000000000000001145f5f56657273696f6e65645f7270632e4d656e7501fd484f01000100",
                MessageHeader::Query(QueryHeader {
                    tag: "__Versioned_rpc.Menu".into(),
                    version: 1,
                    id: 0x00014f48,
                }),
            ),
            (
                concat!(
                    "f80000000000000002fdec57010000feee000a166765745f736f6d655f69",
                    "6e697469616c5f706565727301336765745f7374616765645f6c65646765",
                    "725f6175785f616e645f70656e64696e675f636f696e62617365735f6174",
                    "5f686173680118616e737765725f73796e635f6c65646765725f71756572",
                    "79010c6765745f626573745f746970010c6765745f616e63657374727901",
                    "184765745f7472616e736974696f6e5f6b6e6f776c656467650114676574",
                    "5f7472616e736974696f6e5f636861696e011a6765745f7472616e736974",
                    "696f6e5f636861696e5f70726f6f66010a62616e5f6e6f74696679011067",
                    "65745f65706f63685f6c656467657201"
                ),
                MessageHeader::Response(ResponseHeader { id: 0x000157ec }),
            ),
        ] {
            let s = hex::decode(s).unwrap();
            let mut p = s.as_slice();
            let msg = MessageHeader::read_from_stream(&mut p).unwrap();
            assert_eq!(msg, m);
        }
    }

    #[test]
    fn multiple_messages() {
        let s = hex::decode(concat!(
            "1e0000000000000001145f5f56657273696f6e65645f7270632e4d656e7501fd484f01000100",
            "f80000000000000002fdec57010000feee000a166765745f736f6d655f69",
            "6e697469616c5f706565727301336765745f7374616765645f6c65646765",
            "725f6175785f616e645f70656e64696e675f636f696e62617365735f6174",
            "5f686173680118616e737765725f73796e635f6c65646765725f71756572",
            "79010c6765745f626573745f746970010c6765745f616e63657374727901",
            "184765745f7472616e736974696f6e5f6b6e6f776c656467650114676574",
            "5f7472616e736974696f6e5f636861696e011a6765745f7472616e736974",
            "696f6e5f636861696e5f70726f6f66010a62616e5f6e6f74696679011067",
            "65745f65706f63685f6c656467657201"
        ))
        .unwrap();

        let mut p = s.as_slice();
        for msg in [
            MessageHeader::Query(QueryHeader {
                tag: "__Versioned_rpc.Menu".into(),
                version: 1,
                id: 0x00014f48,
            }),
            MessageHeader::Response(ResponseHeader { id: 0x000157ec }),
        ] {
            assert_eq!(MessageHeader::read_from_stream(&mut p).unwrap(), msg);
        }
    }
}

use std::io::Read;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::versioned::Ver;

pub type Tag = super::string::String;
pub type QueryID = i64;
pub type Sexp = (); // TODO

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, derive_more::From)]
pub struct RpcResult<T, E>(Result<T, E>);

/// Auxiliary type to encode [RpcResult]'s tag.
#[derive(Debug, BinProtRead, BinProtWrite)]
enum Result_ {
    Ok,
    Err,
}

impl<T, E> BinProtRead for RpcResult<T, E>
where
    T: BinProtRead,
    E: BinProtRead,
{
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        Ok(match Result_::binprot_read(r)? {
            Result_::Ok => Ok(T::binprot_read(r)?),
            Result_::Err => Err(E::binprot_read(r)?),
        }
        .into())
    }
}

impl<T, E> BinProtWrite for RpcResult<T, E>
where
    T: BinProtWrite,
    E: BinProtWrite,
{
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        match &self.0 {
            Ok(v) => {
                Result_::Ok.binprot_write(w)?;
                v.binprot_write(w)?;
            }
            Err(e) => {
                Result_::Err.binprot_write(w)?;
                e.binprot_write(w)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, derive_more::From)]
pub struct NeedsLength<T>(T);

impl<T> BinProtRead for NeedsLength<T>
where
    T: BinProtRead,
{
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let _size = binprot::Nat0::binprot_read(r)?.0;
        // Trait function requires r to be ?Sized, so we cannot use `take`
        // use std::io;
        // let mut r = r.take(size);
        // let v = T::binprot_read(&mut r)?;
        // io::copy(&mut r, &mut io::sink())?;
        let v = T::binprot_read(r)?;
        Ok(v.into())
    }
}

impl<T> BinProtWrite for NeedsLength<T>
where
    T: BinProtWrite,
{
    fn binprot_write<W: std::io::Write>(&self, _w: &mut W) -> std::io::Result<()> {
        todo!("need also BinProtSized")
    }
}

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
#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub enum Error {
    Bin_io_exn, //(Sexp),
    Connection_closed,
    Write_error,  //(Sexp),
    Uncaught_exn, //(Sexp),
    Unimplemented_rpc(Tag, Ver),
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
#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub struct Query<T> {
    pub tag: Tag,
    pub version: Ver,
    pub id: QueryID,
    pub data: NeedsLength<T>,
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
#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub struct Response<T> {
    pub id: QueryID,
    pub data: RpcResult<NeedsLength<T>, Error>,
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
#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub enum Message<T> {
    Heartbeat,
    Query(Query<T>),
    Response(Response<T>),
}

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub struct QueryHeader {
    pub tag: Tag,
    pub version: i32,
    pub id: QueryID,
}

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub struct ResponseHeader {
    pub id: QueryID,
}

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, Eq)]
pub enum MessageHeader {
    Heartbeat,
    Query(QueryHeader),
    Response(ResponseHeader),
}

pub trait RpcMethod {
    const NAME: &'static str;
    type Query;
    type Response;
}

/// Reads binable (bin_prot-encoded) value from a stream, handles it and returns
/// a result.
pub trait BinableDecoder {
    type Output;
    fn handle(&self, r: Box<&mut dyn Read>) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use binprot::BinProtRead;
    use binprot_derive::BinProtRead;

    use crate::{
        rpc::{NeedsLength, RpcResult, Tag},
        utils::FromBinProtStream,
        versioned::Ver,
    };

    use super::{Message, MessageHeader, Query, QueryHeader, Response, ResponseHeader};

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

    fn test_message<T>(encoded: &str, decoded: T)
    where
        T: BinProtRead + std::fmt::Debug + PartialEq,
    {
        let s = hex::decode(encoded).unwrap();
        let mut p = s.as_slice();
        let msg = T::read_from_stream(&mut p).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn message() {
        test_message(
            "1e0000000000000001145f5f56657273696f6e65645f7270632e4d656e7501fd484f01000100",
            Message::Query(Query {
                tag: "__Versioned_rpc.Menu".into(),
                version: 1,
                id: 0x00014f48,
                data: ().into(),
            }),
        );

        #[derive(Debug, BinProtRead, PartialEq)]
        struct RpcTagVersion {
            tag: Tag,
            version: Ver,
        }

        type QueryType = Vec<RpcTagVersion>;

        test_message(
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
            Message::<QueryType>::Response(Response {
                id: 0x000157ec,
                data: RpcResult::from(Ok(NeedsLength::from(
                    [
                        "get_some_initial_peers",
                        "get_staged_ledger_aux_and_pending_coinbases_at_hash",
                        "answer_sync_ledger_query",
                        "get_best_tip",
                        "get_ancestry",
                        "Get_transition_knowledge",
                        "get_transition_chain",
                        "get_transition_chain_proof",
                        "ban_notify",
                        "get_epoch_ledger",
                    ]
                    .into_iter()
                    .map(|tag| RpcTagVersion {
                        tag: tag.into(),
                        version: 1,
                    })
                    .collect::<Vec<_>>(),
                ))),
            }),
        );
    }
}

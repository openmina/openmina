pub mod outgoing;

mod p2p_rpc_state;
pub use p2p_rpc_state::*;

mod p2p_rpc_actions;
pub use p2p_rpc_actions::*;

mod p2p_rpc_reducer;
pub use p2p_rpc_reducer::*;

mod p2p_rpc_service;
pub use p2p_rpc_service::*;

use std::io;

use binprot::{BinProtRead, BinProtWrite};
use libp2p::futures::io::{AsyncRead, AsyncReadExt};
use mina_p2p_messages::{
    bigint::BigInt,
    rpc::{
        AnswerSyncLedgerQueryV2, GetBestTipV2, GetTransitionChainV2, GetTransitionKnowledgeV1ForV2,
        VersionedRpcMenuV1,
    },
    rpc_kernel::{QueryHeader, QueryID, Response, ResponseHeader, RpcMethod, RpcResultKind},
    v2::MinaLedgerSyncLedgerAnswerStableV2,
};
use serde::{Deserialize, Serialize};

use crate::PeerId;

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct P2pRpcIdType;
impl shared::requests::RequestIdType for P2pRpcIdType {
    fn request_id_type() -> &'static str {
        "P2pRpcId"
    }
}

pub type P2pRpcId = shared::requests::RequestId<P2pRpcIdType>;
pub type P2pRpcIncomingId = u64;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum P2pRpcOutgoingError {
    ConnectionClosed,
    ProtocolUnsupported,
    ResponseInvalid(P2pRpcResponseInvalidError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcEvent {
    OutgoingResponse(PeerId, P2pRpcId, P2pRpcResponse),
    OutgoingError(PeerId, P2pRpcId, P2pRpcOutgoingError),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum P2pRpcResponseInvalidError {
    UnexpectedResponseKind,
    LedgerHashMismatch { expected: BigInt, found: BigInt },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcRequest {
    MenuGet(<VersionedRpcMenuV1 as RpcMethod>::Query),
    BestTipGet(<GetBestTipV2 as RpcMethod>::Query),
    TransitionKnowledgeGet(<GetTransitionKnowledgeV1ForV2 as RpcMethod>::Query),
    TransitionChainGet(<GetTransitionChainV2 as RpcMethod>::Query),
    LedgerQuery(<AnswerSyncLedgerQueryV2 as RpcMethod>::Query),
}

impl P2pRpcRequest {
    pub fn kind(&self) -> P2pRpcKind {
        match self {
            Self::MenuGet(_) => P2pRpcKind::MenuGet,
            Self::BestTipGet(_) => P2pRpcKind::BestTipGet,
            Self::TransitionKnowledgeGet(_) => P2pRpcKind::TransitionKnowledgeGet,
            Self::TransitionChainGet(_) => P2pRpcKind::TransitionChainGet,
            Self::LedgerQuery(_) => P2pRpcKind::LedgerQuery,
        }
    }

    fn write_msg_impl<T, W>(w: &mut W, id: P2pRpcId, data: &T::Query) -> io::Result<()>
    where
        T: RpcMethod,
        W: io::Write,
    {
        let header = QueryHeader {
            tag: T::NAME.into(),
            version: T::VERSION,
            id: id.counter() as QueryID,
        };
        header.binprot_write(w)?;

        let mut buf = Vec::new();
        data.binprot_write(&mut buf)?;
        binprot::Nat0(buf.len() as u64).binprot_write(w)?;
        w.write_all(&buf)
    }

    pub fn write_msg<W: io::Write>(&self, id: P2pRpcId, w: &mut W) -> io::Result<()> {
        match self {
            Self::MenuGet(data) => Self::write_msg_impl::<VersionedRpcMenuV1, _>(w, id, data),
            Self::BestTipGet(data) => Self::write_msg_impl::<GetBestTipV2, _>(w, id, data),
            Self::TransitionKnowledgeGet(data) => {
                Self::write_msg_impl::<GetTransitionKnowledgeV1ForV2, _>(w, id, data)
            }
            Self::TransitionChainGet(data) => {
                Self::write_msg_impl::<GetTransitionChainV2, _>(w, id, data)
            }
            Self::LedgerQuery(data) => {
                Self::write_msg_impl::<AnswerSyncLedgerQueryV2, _>(w, id, data)
            }
        }
    }

    pub fn validate_response(
        &self,
        resp: &P2pRpcResponse,
    ) -> Result<(), P2pRpcResponseInvalidError> {
        if self.kind() != resp.kind() {
            return Err(P2pRpcResponseInvalidError::UnexpectedResponseKind);
        }
        match self {
            Self::LedgerQuery((expected_ledger_hash, _)) => {
                let P2pRpcResponse::LedgerQuery(resp) = resp else { unreachable!() };
                match &resp.0 {
                    Ok(answer) => match answer {
                        MinaLedgerSyncLedgerAnswerStableV2::AccountWithPath(account, path) => {
                            let hash = snark::calc_merkle_root_hash(account, path);
                            if expected_ledger_hash != &hash {
                                Err(P2pRpcResponseInvalidError::LedgerHashMismatch {
                                    expected: expected_ledger_hash.clone(),
                                    found: hash,
                                })
                            } else {
                                Ok(())
                            }
                        }
                        _ => Ok(()),
                    },
                    Err(_) => Ok(()),
                }
            }
            _ => Ok(()),
        }
    }
}

impl Default for P2pRpcRequest {
    fn default() -> Self {
        Self::MenuGet(())
    }
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum P2pRpcKind {
    MenuGet = 0,
    BestTipGet,
    TransitionKnowledgeGet,
    TransitionChainGet,
    LedgerQuery,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcResponse {
    MenuGet(<VersionedRpcMenuV1 as RpcMethod>::Response),
    BestTipGet(<GetBestTipV2 as RpcMethod>::Response),
    TransitionKnowledgeGet(<GetTransitionKnowledgeV1ForV2 as RpcMethod>::Response),
    TransitionChainGet(<GetTransitionChainV2 as RpcMethod>::Response),
    LedgerQuery(<AnswerSyncLedgerQueryV2 as RpcMethod>::Response),
}

impl P2pRpcResponse {
    pub fn kind(&self) -> P2pRpcKind {
        match self {
            Self::MenuGet(_) => P2pRpcKind::MenuGet,
            Self::BestTipGet(_) => P2pRpcKind::BestTipGet,
            Self::TransitionKnowledgeGet(_) => P2pRpcKind::TransitionKnowledgeGet,
            Self::TransitionChainGet(_) => P2pRpcKind::TransitionChainGet,
            Self::LedgerQuery(_) => P2pRpcKind::LedgerQuery,
        }
    }

    fn write_msg_impl<T, W>(w: &mut W, id: P2pRpcId, data: &T::Response) -> io::Result<()>
    where
        T: RpcMethod,
        W: io::Write,
        // TODO(binier): optimize. extra clone here.
        T::Response: Clone,
    {
        let resp = Response {
            id: id.counter() as QueryID,
            // TODO(binier): optimize. extra clone here.
            data: Ok(data.clone().into()).into(),
        };
        resp.binprot_write(w)
    }

    pub fn write_msg<W: io::Write>(&self, id: P2pRpcId, w: &mut W) -> io::Result<()> {
        match self {
            Self::MenuGet(res) => Self::write_msg_impl::<VersionedRpcMenuV1, _>(w, id, res),
            Self::BestTipGet(res) => Self::write_msg_impl::<GetBestTipV2, _>(w, id, res),
            Self::TransitionKnowledgeGet(res) => {
                Self::write_msg_impl::<GetTransitionKnowledgeV1ForV2, _>(w, id, res)
            }
            Self::TransitionChainGet(res) => {
                Self::write_msg_impl::<GetTransitionChainV2, _>(w, id, res)
            }
            Self::LedgerQuery(res) => {
                Self::write_msg_impl::<AnswerSyncLedgerQueryV2, _>(w, id, res)
            }
        }
    }

    pub async fn async_read_msg<R: AsyncRead + Unpin>(
        kind: P2pRpcKind,
        r: &mut R,
    ) -> Result<Self, binprot::Error> {
        let mut buf = [0; 11];
        r.read_exact(&mut buf).await?;
        let mut b = &buf[..];

        let header = ResponseHeader::binprot_read(&mut b)?;
        let result_kind = RpcResultKind::binprot_read(&mut b)?;
        let payload_len = if b.len() == 9 {
            binprot::Nat0::binprot_read(&mut b)?.0 as usize
        } else {
            let mut buf = [0; 9];
            let n = if b.is_empty() {
                0
            } else {
                let n = b.len();
                (&mut buf[0..n]).clone_from_slice(b);
                b = &[];
                n
            };
            r.read_exact(&mut buf[n..]).await?;
            binprot::Nat0::binprot_read(&mut &buf[..])?.0 as usize
        };
        // TODO(bineir): [SECURITY] limit max len.
        let payload_bytes = {
            let mut buf = vec![0; payload_len];
            let n = b.len().min(payload_len);
            if n > 0 {
                (&mut buf[0..n]).clone_from_slice(&b[0..n]);
            }
            r.read(&mut buf[n..]).await?;
            buf
        };

        if matches!(result_kind, RpcResultKind::Err) {
            let err = mina_p2p_messages::rpc_kernel::Error::binprot_read(&mut &payload_bytes[..])?;
            let err = format!("{:?}", err);
            return Err(binprot::Error::CustomError(err.into()));
        }

        Ok(match kind {
            P2pRpcKind::MenuGet => {
                Self::MenuGet(BinProtRead::binprot_read(&mut &payload_bytes[..])?)
            }
            P2pRpcKind::BestTipGet => {
                Self::BestTipGet(BinProtRead::binprot_read(&mut &payload_bytes[..])?)
            }
            P2pRpcKind::TransitionKnowledgeGet => {
                Self::TransitionKnowledgeGet(BinProtRead::binprot_read(&mut &payload_bytes[..])?)
            }
            P2pRpcKind::TransitionChainGet => {
                Self::TransitionChainGet(BinProtRead::binprot_read(&mut &payload_bytes[..])?)
            }
            P2pRpcKind::LedgerQuery => {
                Self::LedgerQuery(BinProtRead::binprot_read(&mut &payload_bytes[..])?)
            }
        })
    }
}

impl Default for P2pRpcResponse {
    fn default() -> Self {
        Self::MenuGet(vec![])
    }
}

#[test]
fn decode_menu_response() {
    let bytes = [
        5, 0, 254, 238, 0, 10, 22, 103, 101, 116, 95, 115, 111, 109, 101, 95, 105, 110, 105, 116,
        105, 97, 108, 95, 112, 101, 101, 114, 115, 1, 51, 103, 101, 116, 95, 115, 116, 97, 103,
        101, 100, 95, 108, 101, 100, 103, 101, 114, 95, 97, 117, 120, 95, 97, 110, 100, 95, 112,
        101, 110, 100, 105, 110, 103, 95, 99, 111, 105, 110, 98, 97, 115, 101, 115, 95, 97, 116,
        95, 104, 97, 115, 104, 1, 24, 97, 110, 115, 119, 101, 114, 95, 115, 121, 110, 99, 95, 108,
        101, 100, 103, 101, 114, 95, 113, 117, 101, 114, 121, 1, 12, 103, 101, 116, 95, 98, 101,
        115, 116, 95, 116, 105, 112, 1, 12, 103, 101, 116, 95, 97, 110, 99, 101, 115, 116, 114,
        121, 1, 24, 71, 101, 116, 95, 116, 114, 97, 110, 115, 105, 116, 105, 111, 110, 95, 107,
        110, 111, 119, 108, 101, 100, 103, 101, 1, 20, 103, 101, 116, 95, 116, 114, 97, 110, 115,
        105, 116, 105, 111, 110, 95, 99, 104, 97, 105, 110, 1, 26, 103, 101, 116, 95, 116, 114, 97,
        110, 115, 105, 116, 105, 111, 110, 95, 99, 104, 97, 105, 110, 95, 112, 114, 111, 111, 102,
        1, 10, 98, 97, 110, 95, 110, 111, 116, 105, 102, 121, 1, 16, 103, 101, 116, 95, 101, 112,
        111, 99, 104, 95, 108, 101, 100, 103, 101, 114, 1,
    ];
    let mut b = &bytes[..];

    let res = libp2p::futures::executor::block_on(P2pRpcResponse::async_read_msg(
        P2pRpcKind::MenuGet,
        &mut b,
    ));
    dbg!(&res);
    assert!(res.is_ok());
    match res.unwrap() {
        P2pRpcResponse::MenuGet(v) => assert_eq!(v.len(), 10),
        _ => panic!("unexpected decoded result"),
    };
}

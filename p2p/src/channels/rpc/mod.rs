mod p2p_channels_rpc_state;
pub use p2p_channels_rpc_state::*;

mod p2p_channels_rpc_actions;
pub use p2p_channels_rpc_actions::*;

mod p2p_channels_rpc_reducer;
pub use p2p_channels_rpc_reducer::*;

mod p2p_channels_rpc_effects;
pub use p2p_channels_rpc_effects::*;

use std::io;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::{
    common::LedgerHashV1,
    rpc::{AnswerSyncLedgerQueryV2, GetBestTipV2, ProofCarryingDataStableV1},
    rpc_kernel::{
        QueryHeader, QueryID, Response, ResponseHeader, RpcMethod, RpcResult, RpcResultKind,
    },
    v2::{
        MinaBaseStateBodyHashStableV1, MinaLedgerSyncLedgerAnswerStableV2,
        MinaLedgerSyncLedgerQueryStableV1,
    },
};
use serde::{Deserialize, Serialize};
use shared::block::ArcBlock;

pub type P2pRpcId = u32;

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum RpcChannelMsg {
    Request(P2pRpcId, P2pRpcRequest),
    Response(P2pRpcId, Option<P2pRpcResponse>),
}

impl RpcChannelMsg {
    pub fn request_id(&self) -> P2pRpcId {
        match self {
            Self::Request(id, _) => *id,
            Self::Response(id, _) => *id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum P2pRpcKind {
    BestTipWithProofGet,
    LedgerQuery,
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcRequest {
    BestTipWithProofGet,
    LedgerQuery(LedgerHashV1, MinaLedgerSyncLedgerQueryStableV1),
}

impl P2pRpcRequest {
    pub fn kind(&self) -> P2pRpcKind {
        match self {
            Self::BestTipWithProofGet => P2pRpcKind::BestTipWithProofGet,
            Self::LedgerQuery(..) => P2pRpcKind::LedgerQuery,
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
            id: id as QueryID,
        };
        header.binprot_write(w)?;

        let mut buf = Vec::new();
        data.binprot_write(&mut buf)?;
        binprot::Nat0(buf.len() as u64).binprot_write(w)?;
        w.write_all(&buf)
    }

    pub fn write_msg<W: io::Write>(self, id: P2pRpcId, w: &mut W) -> io::Result<()> {
        match self {
            Self::BestTipWithProofGet => Self::write_msg_impl::<GetBestTipV2, _>(w, id, &()),
            Self::LedgerQuery(ledger_hash, query) => {
                Self::write_msg_impl::<AnswerSyncLedgerQueryV2, _>(w, id, &(ledger_hash, query))
            }
        }
    }
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct BestTipWithProof {
    pub best_tip: ArcBlock,
    pub proof: (Vec<MinaBaseStateBodyHashStableV1>, ArcBlock),
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcResponse {
    BestTipWithProofGet(BestTipWithProof),
    LedgerQuery(MinaLedgerSyncLedgerAnswerStableV2),
}

impl P2pRpcResponse {
    pub fn kind(&self) -> P2pRpcKind {
        match self {
            Self::BestTipWithProofGet(_) => P2pRpcKind::BestTipWithProofGet,
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
            id: id as QueryID,
            // TODO(binier): optimize. extra clone here.
            data: Ok(data.clone().into()).into(),
        };
        resp.binprot_write(w)
    }

    pub fn write_msg<W: io::Write>(self, id: P2pRpcId, w: &mut W) -> io::Result<()> {
        match self {
            Self::BestTipWithProofGet(res) => Self::write_msg_impl::<GetBestTipV2, _>(
                w,
                id,
                &Some(ProofCarryingDataStableV1 {
                    data: (*res.best_tip).clone(),
                    proof: (res.proof.0, (*res.proof.1).clone()),
                }),
            ),
            Self::LedgerQuery(res) => {
                Self::write_msg_impl::<AnswerSyncLedgerQueryV2, _>(w, id, &RpcResult(Ok(res)))
            }
        }
    }

    pub fn read_msg<R: io::Read>(
        kind: P2pRpcKind,
        r: &mut R,
    ) -> Result<Option<Self>, binprot::Error> {
        let header = ResponseHeader::binprot_read(r)?;
        let result_kind = RpcResultKind::binprot_read(r)?;

        if matches!(result_kind, RpcResultKind::Err) {
            let err = mina_p2p_messages::rpc_kernel::Error::binprot_read(r)?;
            let err = format!("{:?}", err);
            return Err(binprot::Error::CustomError(err.into()));
        }
        let _payload_len = binprot::Nat0::binprot_read(r)?.0 as usize;

        Ok(match kind {
            P2pRpcKind::BestTipWithProofGet => {
                let resp: <GetBestTipV2 as RpcMethod>::Response = BinProtRead::binprot_read(r)?;
                resp.map(|resp| {
                    Self::BestTipWithProofGet(BestTipWithProof {
                        best_tip: resp.data.into(),
                        proof: (resp.proof.0, resp.proof.1.into()),
                    })
                })
            }
            P2pRpcKind::LedgerQuery => {
                let resp: <AnswerSyncLedgerQueryV2 as RpcMethod>::Response =
                    BinProtRead::binprot_read(r)?;
                match resp.0 {
                    // TODO(binier): if err says not found in ledger or
                    // ledger not available, we should return None.
                    Err(err) => return Err(binprot::Error::CustomError(format!("{err:?}").into())),
                    Ok(resp) => Some(Self::LedgerQuery(resp)),
                }
            }
        })
    }
}

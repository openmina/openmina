mod p2p_channels_rpc_state;
pub use p2p_channels_rpc_state::*;

mod p2p_channels_rpc_actions;
pub use p2p_channels_rpc_actions::*;

mod p2p_channels_rpc_reducer;
pub use p2p_channels_rpc_reducer::*;

mod p2p_channels_rpc_effects;
pub use p2p_channels_rpc_effects::*;

use std::{io, sync::Arc, time::Duration};

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::{
    rpc::{
        AnswerSyncLedgerQueryV2, GetBestTipV2, GetStagedLedgerAuxAndPendingCoinbasesAtHashV2,
        GetTransitionChainV2, ProofCarryingDataStableV1,
    },
    rpc_kernel::{
        QueryHeader, QueryID, Response, ResponseHeader, RpcMethod, RpcResult, RpcResultKind,
    },
    v2::{
        LedgerHash, MinaBaseLedgerHash0StableV1, MinaBasePendingCoinbaseStableV2,
        MinaBaseStateBodyHashStableV1, MinaLedgerSyncLedgerAnswerStableV2,
        MinaLedgerSyncLedgerQueryStableV1, MinaStateProtocolStateValueStableV2, StateHash,
        TransactionSnarkScanStateStableV2,
    },
};
use serde::{Deserialize, Serialize};
use shared::{block::ArcBlock, snark::Snark, snark_job_id::SnarkJobId};

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
    BestTipWithProof,
    LedgerQuery,
    StagedLedgerAuxAndPendingCoinbasesAtBlock,
    Block,
    Snark,
}

impl P2pRpcKind {
    pub fn timeout(self) -> Option<Duration> {
        match self {
            Self::BestTipWithProof => Some(Duration::from_secs(10)),
            Self::LedgerQuery => Some(Duration::from_secs(2)),
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock => Some(Duration::from_secs(120)),
            Self::Block => Some(Duration::from_secs(5)),
            Self::Snark => Some(Duration::from_secs(5)),
        }
    }

    pub fn supported_by_libp2p(self) -> bool {
        match self {
            Self::BestTipWithProof => true,
            Self::LedgerQuery => true,
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock => true,
            Self::Block => true,
            Self::Snark => false,
        }
    }
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcRequest {
    BestTipWithProof,
    LedgerQuery(LedgerHash, MinaLedgerSyncLedgerQueryStableV1),
    StagedLedgerAuxAndPendingCoinbasesAtBlock(StateHash),
    Block(StateHash),
    Snark(SnarkJobId),
}

impl P2pRpcRequest {
    pub fn kind(&self) -> P2pRpcKind {
        match self {
            Self::BestTipWithProof => P2pRpcKind::BestTipWithProof,
            Self::LedgerQuery(..) => P2pRpcKind::LedgerQuery,
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock(_) => {
                P2pRpcKind::StagedLedgerAuxAndPendingCoinbasesAtBlock
            }
            Self::Block(_) => P2pRpcKind::Block,
            Self::Snark(_) => P2pRpcKind::Snark,
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
            Self::BestTipWithProof => Self::write_msg_impl::<GetBestTipV2, _>(w, id, &()),
            Self::LedgerQuery(ledger_hash, query) => Self::write_msg_impl::<
                AnswerSyncLedgerQueryV2,
                _,
            >(
                w, id, &(ledger_hash.0.clone(), query)
            ),
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock(block_hash) => {
                Self::write_msg_impl::<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2, _>(
                    w,
                    id,
                    &(block_hash.0.clone()),
                )
            }
            Self::Block(hash) => {
                Self::write_msg_impl::<GetTransitionChainV2, _>(w, id, &vec![hash.0.clone()])
            }
            Self::Snark(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "rpc not supported by ocaml node",
            )),
        }
    }
}

impl Default for P2pRpcRequest {
    fn default() -> Self {
        Self::BestTipWithProof
    }
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct BestTipWithProof {
    pub best_tip: ArcBlock,
    pub proof: (Vec<MinaBaseStateBodyHashStableV1>, ArcBlock),
}

/// Pieces required to reconstruct staged ledger from snarked ledger.
#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct StagedLedgerAuxAndPendingCoinbases {
    pub scan_state: TransactionSnarkScanStateStableV2,
    pub staged_ledger_hash: LedgerHash,
    pub pending_coinbase: MinaBasePendingCoinbaseStableV2,
    pub needed_blocks: Vec<MinaStateProtocolStateValueStableV2>,
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcResponse {
    BestTipWithProof(BestTipWithProof),
    LedgerQuery(MinaLedgerSyncLedgerAnswerStableV2),
    StagedLedgerAuxAndPendingCoinbasesAtBlock(Arc<StagedLedgerAuxAndPendingCoinbases>),
    Block(ArcBlock),
    Snark(Snark),
}

impl P2pRpcResponse {
    pub fn kind(&self) -> P2pRpcKind {
        match self {
            Self::BestTipWithProof(_) => P2pRpcKind::BestTipWithProof,
            Self::LedgerQuery(_) => P2pRpcKind::LedgerQuery,
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock(_) => {
                P2pRpcKind::StagedLedgerAuxAndPendingCoinbasesAtBlock
            }
            Self::Block(_) => P2pRpcKind::Block,
            Self::Snark(_) => P2pRpcKind::Snark,
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
            Self::BestTipWithProof(res) => Self::write_msg_impl::<GetBestTipV2, _>(
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
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock(res) => {
                let res = Arc::try_unwrap(res).unwrap_or_else(|res| (*res).clone());
                let res = (
                    res.scan_state,
                    res.staged_ledger_hash.0.clone(),
                    res.pending_coinbase,
                    res.needed_blocks,
                );
                Self::write_msg_impl::<GetStagedLedgerAuxAndPendingCoinbasesAtHashV2, _>(
                    w,
                    id,
                    &Some(res),
                )
            }
            Self::Block(res) => {
                let res = Arc::try_unwrap(res).unwrap_or_else(|res| (*res).clone());
                Self::write_msg_impl::<GetTransitionChainV2, _>(w, id, &Some(vec![res]))
            }
            Self::Snark(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "rpc not supported by ocaml node",
            )),
        }
    }

    pub fn read_msg<R: io::Read>(
        kind: P2pRpcKind,
        r: &mut R,
    ) -> Result<Option<Self>, binprot::Error> {
        let _header = ResponseHeader::binprot_read(r)?;
        let result_kind = RpcResultKind::binprot_read(r)?;

        if matches!(result_kind, RpcResultKind::Err) {
            let err = mina_p2p_messages::rpc_kernel::Error::binprot_read(r)?;
            let err = format!("{:?}", err);
            return Err(binprot::Error::CustomError(err.into()));
        }
        let _payload_len = binprot::Nat0::binprot_read(r)?.0 as usize;

        Ok(match kind {
            P2pRpcKind::BestTipWithProof => {
                let resp: <GetBestTipV2 as RpcMethod>::Response = BinProtRead::binprot_read(r)?;
                resp.map(|resp| {
                    Self::BestTipWithProof(BestTipWithProof {
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
            P2pRpcKind::StagedLedgerAuxAndPendingCoinbasesAtBlock => {
                let resp: <GetStagedLedgerAuxAndPendingCoinbasesAtHashV2 as RpcMethod>::Response =
                    BinProtRead::binprot_read(r)?;
                resp.map(|resp| StagedLedgerAuxAndPendingCoinbases {
                    scan_state: resp.0,
                    staged_ledger_hash: MinaBaseLedgerHash0StableV1(resp.1).into(),
                    pending_coinbase: resp.2,
                    needed_blocks: resp.3,
                })
                .map(Arc::from)
                .map(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock)
            }
            P2pRpcKind::Block => {
                let resp: <GetTransitionChainV2 as RpcMethod>::Response =
                    BinProtRead::binprot_read(r)?;
                resp.and_then(|blocks| blocks.into_iter().next())
                    .map(|block| block.into())
                    .map(P2pRpcResponse::Block)
            }
            P2pRpcKind::Snark => None,
        })
    }
}

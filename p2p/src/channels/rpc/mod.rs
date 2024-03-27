mod p2p_channels_rpc_state;
pub use p2p_channels_rpc_state::*;

mod p2p_channels_rpc_actions;
pub use p2p_channels_rpc_actions::*;

mod p2p_channels_rpc_reducer;

mod p2p_channels_rpc_effects;

use std::{sync::Arc, time::Duration};

use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::{
    rpc,
    rpc_kernel::{
        NeedsLength, QueryHeader, QueryPayload, ResponseHeader, ResponsePayload, RpcMethod,
        RpcResult,
    },
    v2::{
        LedgerHash, MerkleAddressBinableArgStableV1, MinaBasePendingCoinbaseStableV2,
        MinaBaseStateBodyHashStableV1, MinaLedgerSyncLedgerAnswerStableV2,
        MinaLedgerSyncLedgerQueryStableV1, MinaStateProtocolStateValueStableV2, StateHash,
        TransactionSnarkScanStateStableV2,
    },
};
use openmina_core::{
    block::ArcBlock,
    snark::{Snark, SnarkJobId},
};
use serde::{Deserialize, Serialize};

use crate::{connection::outgoing::P2pConnectionOutgoingInitOpts, Data, P2pTimeouts};

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
    InitialPeers,
}

impl P2pRpcKind {
    pub fn timeout(self, config: &P2pTimeouts) -> Option<Duration> {
        match self {
            Self::BestTipWithProof => config.best_tip_with_proof,
            Self::LedgerQuery => config.ledger_query,
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock => {
                config.staged_ledger_aux_and_pending_coinbases_at_block
            }
            Self::Block => config.block,
            Self::Snark => config.snark,
            Self::InitialPeers => config.initial_peers,
        }
    }

    pub fn supported_by_libp2p(self) -> bool {
        match self {
            Self::BestTipWithProof => true,
            Self::LedgerQuery => true,
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock => true,
            Self::Block => true,
            Self::Snark => false,
            Self::InitialPeers => true,
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
    InitialPeers,
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
            Self::InitialPeers => P2pRpcKind::InitialPeers,
        }
    }
}

impl Default for P2pRpcRequest {
    fn default() -> Self {
        Self::BestTipWithProof
    }
}

fn addr_to_str(
    MerkleAddressBinableArgStableV1(mina_p2p_messages::number::Number(length), byte_string): &MerkleAddressBinableArgStableV1,
) -> String {
    let addr = byte_string
        .as_ref()
        .iter()
        .copied()
        .flat_map(|byte| {
            (0..8)
                .map(move |b| byte & (1 << (7 - b)) != 0)
                .map(|b| if b { '1' } else { '0' })
        })
        .take(*length as usize)
        .collect::<String>();

    format!("depth: {length}, addr: {addr}")
}

impl std::fmt::Display for P2pRpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind())?;
        match self {
            Self::BestTipWithProof => Ok(()),
            Self::LedgerQuery(ledger_hash, query) => {
                match query {
                    MinaLedgerSyncLedgerQueryStableV1::NumAccounts => write!(f, ", NumAccounts, ")?,
                    MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(addr) => {
                        write!(f, ", ChildHashes, {}, ", addr_to_str(addr))?
                    }
                    MinaLedgerSyncLedgerQueryStableV1::WhatContents(addr) => {
                        write!(f, ", ChildContents, {}, ", addr_to_str(addr))?
                    }
                }
                write!(f, "ledger: {ledger_hash}")
            }
            Self::StagedLedgerAuxAndPendingCoinbasesAtBlock(block_hash)
            | Self::Block(block_hash) => {
                write!(f, ", {block_hash}")
            }
            Self::Snark(job_id) => {
                write!(f, ", {job_id}")
            }
            Self::InitialPeers => Ok(()),
        }
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
    InitialPeers(Vec<P2pConnectionOutgoingInitOpts>),
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
            Self::InitialPeers(_) => P2pRpcKind::InitialPeers,
        }
    }
}

fn internal_response_into_libp2p(
    response: P2pRpcResponse,
    id: P2pRpcId,
) -> Option<(ResponseHeader, Data)> {
    use binprot::BinProtWrite;

    match response {
        P2pRpcResponse::BestTipWithProof(r) => {
            type Method = rpc::GetBestTipV2;
            type Payload = ResponsePayload<<Method as RpcMethod>::Response>;

            let BestTipWithProof {
                best_tip,
                proof: (middle, block),
            } = r;

            let r = RpcResult(Ok(NeedsLength(Some(rpc::ProofCarryingDataStableV1 {
                data: best_tip.as_ref().clone(),
                proof: (middle, block.as_ref().clone()),
            }))));

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&r, &mut v).unwrap_or_default();
            Some((ResponseHeader { id: id as _ }, v.into()))
        }
        P2pRpcResponse::LedgerQuery(answer) => {
            type Method = rpc::AnswerSyncLedgerQueryV2;
            type Payload = ResponsePayload<<Method as RpcMethod>::Response>;

            let r = RpcResult(Ok(NeedsLength(RpcResult(Ok(answer)))));

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&r, &mut v).unwrap_or_default();
            Some((ResponseHeader { id: id as _ }, v.into()))
        }
        P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock(staged_ledger_info) => {
            type Method = rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
            type Payload = ResponsePayload<<Method as RpcMethod>::Response>;

            let StagedLedgerAuxAndPendingCoinbases {
                scan_state,
                staged_ledger_hash,
                pending_coinbase,
                needed_blocks,
            } = staged_ledger_info.as_ref().clone();

            let hash = staged_ledger_hash.inner().0.clone();

            let r = RpcResult(Ok(NeedsLength(Some((
                scan_state,
                hash,
                pending_coinbase,
                needed_blocks,
            )))));

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&r, &mut v).unwrap_or_default();
            Some((ResponseHeader { id: id as _ }, v.into()))
        }
        P2pRpcResponse::Block(block) => {
            type Method = rpc::GetTransitionChainV2;
            type Payload = ResponsePayload<<Method as RpcMethod>::Response>;

            let r = RpcResult(Ok(NeedsLength(Some(vec![block.as_ref().clone()]))));

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&r, &mut v).unwrap_or_default();
            Some((ResponseHeader { id: id as _ }, v.into()))
        }
        P2pRpcResponse::Snark(_) => {
            // should use gossipsub to broadcast
            None
        }
        P2pRpcResponse::InitialPeers(peers) => {
            type Method = rpc::GetSomeInitialPeersV1ForV2;
            type Payload = ResponsePayload<<Method as RpcMethod>::Response>;

            let r = peers
                .into_iter()
                .filter_map(|peer| peer.try_into_mina_rpc())
                .collect();
            let r = RpcResult(Ok(NeedsLength(r)));

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&r, &mut v).unwrap_or_default();
            Some((ResponseHeader { id: id as _ }, v.into()))
        }
    }
}

fn internal_request_into_libp2p(
    request: P2pRpcRequest,
    id: P2pRpcId,
) -> Option<(QueryHeader, Data)> {
    use binprot::BinProtWrite;

    match request {
        P2pRpcRequest::BestTipWithProof => {
            type Method = rpc::GetBestTipV2;
            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&NeedsLength(()), &mut v).unwrap_or_default();
            Some((
                QueryHeader {
                    tag: Method::NAME.into(),
                    version: Method::VERSION,
                    id: id as _,
                },
                v.into(),
            ))
        }
        P2pRpcRequest::LedgerQuery(hash, q) => {
            type Method = rpc::AnswerSyncLedgerQueryV2;
            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&NeedsLength((hash.0.clone(), q)), &mut v)
                .unwrap_or_default();
            Some((
                QueryHeader {
                    tag: Method::NAME.into(),
                    version: Method::VERSION,
                    id: id as _,
                },
                v.into(),
            ))
        }
        P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(hash) => {
            type Method = rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&NeedsLength(hash.0.clone()), &mut v)
                .unwrap_or_default();
            Some((
                QueryHeader {
                    tag: Method::NAME.into(),
                    version: Method::VERSION,
                    id: id as _,
                },
                v.into(),
            ))
        }
        P2pRpcRequest::Block(hash) => {
            type Method = rpc::GetTransitionChainV2;
            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&NeedsLength(vec![hash.0.clone()]), &mut v)
                .unwrap_or_default();
            Some((
                QueryHeader {
                    tag: Method::NAME.into(),
                    version: Method::VERSION,
                    id: id as _,
                },
                v.into(),
            ))
        }
        P2pRpcRequest::Snark(hash) => {
            let _ = hash;
            // libp2p cannot fulfill this request
            None
        }
        P2pRpcRequest::InitialPeers => {
            type Method = rpc::GetSomeInitialPeersV1ForV2;
            type Payload = QueryPayload<<Method as RpcMethod>::Query>;

            let mut v = vec![];
            <Payload as BinProtWrite>::binprot_write(&NeedsLength(()), &mut v).unwrap_or_default();
            Some((
                QueryHeader {
                    tag: Method::NAME.into(),
                    version: Method::VERSION,
                    id: id as _,
                },
                v.into(),
            ))
        }
    }
}

use std::collections::BTreeMap;
use std::sync::Arc;

use mina_p2p_messages::v2::{MinaStateProtocolStateValueStableV2, StateHash};
use openmina_core::block::ArcBlockWithHash;
use p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;

use super::{
    PeerStagedLedgerPartsFetchError, StagedLedgerAuxAndPendingCoinbasesValid,
    StagedLedgerAuxAndPendingCoinbasesValidated,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerStagedState {
    /// Fetching pieces required to reconstruct staged ledger from
    /// snarked ledger.
    PartsFetchPending {
        time: Timestamp,
        block: ArcBlockWithHash,
        attempts: BTreeMap<PeerId, PeerStagedLedgerPartsFetchState>,
    },
    /// Fetched pieces required to reconstruct staged ledger from
    /// snarked ledger.
    PartsFetchSuccess {
        time: Timestamp,
        block: ArcBlockWithHash,
        parts: Arc<StagedLedgerAuxAndPendingCoinbasesValid>,
    },
    ReconstructEmpty {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    ReconstructPending {
        time: Timestamp,
        block: ArcBlockWithHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    },
    ReconstructError {
        time: Timestamp,
        block: ArcBlockWithHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
        error: String,
    },
    ReconstructSuccess {
        time: Timestamp,
        block: ArcBlockWithHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    },
    Success {
        time: Timestamp,
        block: ArcBlockWithHash,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerStagedLedgerPartsFetchState {
    Pending {
        time: Timestamp,
        rpc_id: P2pRpcId,
    },
    Error {
        time: Timestamp,
        rpc_id: P2pRpcId,
        error: PeerStagedLedgerPartsFetchError,
    },
    Success {
        time: Timestamp,
        parts: StagedLedgerAuxAndPendingCoinbasesValidated,
    },
    Invalid {
        time: Timestamp,
    },
    Valid {
        time: Timestamp,
        parts: Arc<StagedLedgerAuxAndPendingCoinbasesValid>,
    },
}

impl TransitionFrontierSyncLedgerStagedState {
    pub fn pending(time: Timestamp, block: ArcBlockWithHash) -> Self {
        Self::PartsFetchPending {
            time,
            block,
            attempts: Default::default(),
        }
    }

    pub fn block(&self) -> &ArcBlockWithHash {
        match self {
            Self::PartsFetchPending { block, .. } => block,
            Self::PartsFetchSuccess { block, .. } => block,
            Self::ReconstructEmpty { block, .. } => block,
            Self::ReconstructPending { block, .. } => block,
            Self::ReconstructError { block, .. } => block,
            Self::ReconstructSuccess { block, .. } => block,
            Self::Success { block, .. } => block,
        }
    }

    pub fn block_with_parts(
        &self,
    ) -> Option<(
        &ArcBlockWithHash,
        Option<&Arc<StagedLedgerAuxAndPendingCoinbases>>,
    )> {
        Some(match self {
            Self::PartsFetchSuccess { block, parts, .. } => (block, Some(parts)),
            Self::ReconstructEmpty { block, .. } => (block, None),
            _ => return None,
        })
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn fetch_attempts(&self) -> Option<&BTreeMap<PeerId, PeerStagedLedgerPartsFetchState>> {
        match self {
            Self::PartsFetchPending { attempts, .. } => Some(attempts),
            _ => None,
        }
    }

    pub fn filter_available_peers<'a>(
        &'a self,
        iter: impl 'a + Iterator<Item = (PeerId, P2pRpcId)>,
    ) -> impl 'a + Iterator<Item = (PeerId, P2pRpcId)> {
        let attempts = self.fetch_attempts();
        iter.filter(move |(peer_id, _)| {
            attempts.map_or(false, |attempts| {
                !attempts.contains_key(&peer_id)
                    && (attempts.is_empty() || attempts.iter().all(|(_, s)| s.is_error()))
            })
        })
    }

    pub fn parts_fetch_rpc_id(&self, peer_id: &PeerId) -> Option<P2pRpcId> {
        self.fetch_attempts()?.get(peer_id)?.fetch_pending_rpc_id()
    }
}

impl PeerStagedLedgerPartsFetchState {
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Invalid { .. })
    }

    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid { .. })
    }

    pub fn fetch_pending_rpc_id(&self) -> Option<P2pRpcId> {
        match self {
            Self::Pending { rpc_id, .. } => Some(*rpc_id),
            _ => None,
        }
    }
}

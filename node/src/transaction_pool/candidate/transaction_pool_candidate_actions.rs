use openmina_core::transaction::{TransactionHash, TransactionInfo, TransactionWithHash};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;

use super::TransactionPoolCandidateState;

pub type TransactionPoolCandidateActionWithMeta =
    redux::ActionWithMeta<TransactionPoolCandidateAction>;
pub type TransactionPoolCandidateActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransactionPoolCandidateAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum TransactionPoolCandidateAction {
    InfoReceived {
        peer_id: PeerId,
        info: TransactionInfo,
    },
    #[action_event(level = trace)]
    FetchAll,
    FetchInit {
        peer_id: PeerId,
        hash: TransactionHash,
    },
    FetchPending {
        peer_id: PeerId,
        hash: TransactionHash,
        rpc_id: P2pRpcId,
    },
    FetchError {
        peer_id: PeerId,
        hash: TransactionHash,
    },
    FetchSuccess {
        peer_id: PeerId,
        transaction: TransactionWithHash,
    },
    #[action_event(level = trace)]
    VerifyNext,
    VerifyPending {
        peer_id: PeerId,
        transaction_hashes: Vec<TransactionHash>,
        verify_id: (),
    },
    VerifyError {
        peer_id: PeerId,
        verify_id: (),
    },
    VerifySuccess {
        peer_id: PeerId,
        verify_id: (),
    },
    PeerPrune {
        peer_id: PeerId,
    },
}

impl redux::EnablingCondition<crate::State> for TransactionPoolCandidateAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            TransactionPoolCandidateAction::InfoReceived { info, .. } => {
                !state.transaction_pool.contains(&info.hash)
                    && !state.transaction_pool.candidates.contains(&info.hash)
            }
            TransactionPoolCandidateAction::FetchAll => state.p2p.ready().is_some(),
            TransactionPoolCandidateAction::FetchInit { peer_id, hash } => {
                let is_peer_available = state
                    .p2p
                    .get_ready_peer(peer_id)
                    .map_or(false, |peer| peer.channels.rpc.can_send_request());
                is_peer_available
                    && state
                        .transaction_pool
                        .candidates
                        .get(*peer_id, hash)
                        .map_or(false, |s| {
                            matches!(s, TransactionPoolCandidateState::InfoReceived { .. })
                        })
            }
            TransactionPoolCandidateAction::FetchPending { peer_id, hash, .. } => state
                .transaction_pool
                .candidates
                .get(*peer_id, hash)
                .map_or(false, |s| {
                    matches!(s, TransactionPoolCandidateState::InfoReceived { .. })
                }),
            TransactionPoolCandidateAction::FetchError { peer_id, hash } => state
                .transaction_pool
                .candidates
                .get(*peer_id, hash)
                .is_some(),
            TransactionPoolCandidateAction::FetchSuccess {
                peer_id,
                transaction,
            } => state
                .transaction_pool
                .candidates
                .get(*peer_id, transaction.hash())
                .is_some(),
            TransactionPoolCandidateAction::VerifyNext => true,
            TransactionPoolCandidateAction::VerifyPending {
                peer_id,
                transaction_hashes,
                ..
            } => {
                !transaction_hashes.is_empty()
                    && state
                        .transaction_pool
                        .candidates
                        .candidates_from_peer_with_hashes(*peer_id, transaction_hashes)
                        .all(|(_, state)| {
                            matches!(state, Some(TransactionPoolCandidateState::Received { .. }))
                        })
            }
            TransactionPoolCandidateAction::VerifyError { .. } => {
                // TODO(binier)
                true
            }
            TransactionPoolCandidateAction::VerifySuccess { .. } => {
                // TODO(binier)
                true
            }
            TransactionPoolCandidateAction::PeerPrune { peer_id } => {
                state
                    .transaction_pool
                    .candidates
                    .peer_transaction_count(peer_id)
                    > 0
            }
        }
    }
}

use crate::transaction_pool::TransactionPoolAction;

impl From<TransactionPoolCandidateAction> for crate::Action {
    fn from(value: TransactionPoolCandidateAction) -> Self {
        Self::TransactionPool(TransactionPoolAction::Candidate(value))
    }
}

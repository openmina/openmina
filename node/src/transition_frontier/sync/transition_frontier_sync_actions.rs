use mina_p2p_messages::v2::StateHash;
use openmina_core::block::ArcBlockWithHash;
use openmina_core::consensus::consensus_take;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::ledger::write::CommitResult;
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;
use crate::transition_frontier::sync::TransitionFrontierSyncLedgerPending;
use crate::TransitionFrontierAction;

use super::ledger::{
    SyncLedgerTarget, TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerState,
};
use super::{PeerBlockFetchError, TransitionFrontierSyncState};

pub type TransitionFrontierSyncActionWithMeta = redux::ActionWithMeta<TransitionFrontierSyncAction>;
pub type TransitionFrontierSyncActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierSyncAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum TransitionFrontierSyncAction {
    /// Set transition frontier target to new best tip (for still unsynced frontiers)
    #[action_event(level = info, fields(
        block_hash = display(&best_tip.hash),
        root_block_hash = display(&root_block.hash),
    ))]
    Init {
        best_tip: ArcBlockWithHash,
        root_block: ArcBlockWithHash,
        blocks_inbetween: Vec<StateHash>,
    },
    /// Set sync target to a new best tip
    #[action_event(level = info, fields(
        new_best_tip_hash = display(&best_tip.hash),
        new_best_tip_height = best_tip.height(),
        new_root_block_hash = display(&root_block.hash),
        new_root_snarked_ledger_hash = display(root_block.snarked_ledger_hash()),
        new_root_staged_ledger_hash = display(root_block.staged_ledger_hash()),
    ))]
    BestTipUpdate {
        best_tip: ArcBlockWithHash,
        root_block: ArcBlockWithHash,
        blocks_inbetween: Vec<StateHash>,
    },
    /// Staking Ledger sync is pending
    #[action_event(level = info)]
    LedgerStakingPending,
    /// Staking Ledger sync was successful
    #[action_event(level = info)]
    LedgerStakingSuccess,
    /// Next Epoch Ledger sync is pending
    #[action_event(level = info)]
    LedgerNextEpochPending,
    /// Next Epoch Ledger sync was successful
    #[action_event(level = info)]
    LedgerNextEpochSuccess,
    /// Transition frontier Root Ledger sync is pending
    #[action_event(level = info)]
    LedgerRootPending,
    /// Transition frontier Root Ledger sync was successful
    #[action_event(level = info)]
    LedgerRootSuccess,
    BlocksPending,
    BlocksPeersQuery,
    BlocksPeerQueryInit {
        hash: StateHash,
        peer_id: PeerId,
    },
    BlocksPeerQueryRetry {
        hash: StateHash,
        peer_id: PeerId,
    },
    BlocksPeerQueryPending {
        hash: StateHash,
        peer_id: PeerId,
        rpc_id: P2pRpcId,
    },
    BlocksPeerQueryError {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        error: PeerBlockFetchError,
    },
    BlocksPeerQuerySuccess {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        response: ArcBlockWithHash,
    },
    BlocksFetchSuccess {
        hash: StateHash,
    },
    BlocksNextApplyInit,
    BlocksNextApplyPending {
        hash: StateHash,
    },
    BlocksNextApplySuccess {
        hash: StateHash,
    },
    BlocksSuccess,
    CommitInit,
    CommitPending,
    CommitSuccess {
        result: CommitResult,
    },
    /// Synchronization to a target ledger
    Ledger(TransitionFrontierSyncLedgerAction),
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            TransitionFrontierSyncAction::Init { best_tip, .. } => {
                !state.transition_frontier.sync.is_pending()
                    && !state.transition_frontier.sync.is_synced()
                    && state
                        .transition_frontier
                        .best_tip()
                        .map_or(true, |tip| best_tip.hash != tip.hash)
                    && state
                        .consensus
                        .best_tip()
                        .map_or(false, |tip| &best_tip.hash == tip.hash)
            }
            TransitionFrontierSyncAction::BestTipUpdate { best_tip, .. } => {
                (state.transition_frontier.sync.is_pending() || state.transition_frontier.sync.is_synced())
                    && !matches!(&state.transition_frontier.sync, TransitionFrontierSyncState::CommitPending { .. } | TransitionFrontierSyncState::CommitSuccess { .. })
                && state
                    .transition_frontier
                    .best_tip()
                    .map_or(true, |tip| best_tip.hash != tip.hash)
                && state
                    .transition_frontier
                    .sync
                    .best_tip()
                    .map_or(true, |tip| best_tip.hash != tip.hash)
                // TODO(binier): TMP. we shouldn't need to check consensus here.
                && state
                    .transition_frontier
                    .sync
                    .best_tip()
                    .or(state.transition_frontier.best_tip())
                    .map_or(false, |tip| {
                        if tip.is_genesis() && best_tip.height() > tip.height() {
                            // TODO(binier): once genesis blocks are same, uncomment below.
                            // tip.hash() == &best_tip.header().protocol_state.body.genesis_state_hash
                            true
                        } else {
                            consensus_take(tip.consensus_state(), best_tip.consensus_state(), tip.hash(), best_tip.hash())
                        }
                    })
                // Don't sync to best tip if we are in the middle of producing
                // a block unless that best tip candidate is better consensus-wise
                // than the one that we are producing.
                //
                // Otherwise other block producers might spam the network
                // with blocks that are better than current best tip, yet
                // inferior to the block that we are producing and we can't
                // let that get in the way of us producing a block.
                && state.block_producer.producing_won_slot()
                    .filter(|_| !state.block_producer.is_me(best_tip.producer()))
                    // TODO(binier): check if candidate best tip is short or
                    // long range fork and based on that compare slot that
                    // we are producing.
                    .map_or(true, |won_slot| won_slot < best_tip)
            }
            TransitionFrontierSyncAction::LedgerStakingPending => {
                matches!(
                    state.transition_frontier.sync,
                    TransitionFrontierSyncState::Init { .. }
                )
            }
            TransitionFrontierSyncAction::LedgerStakingSuccess => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::StakingLedgerPending(
                    TransitionFrontierSyncLedgerPending {
                        ledger: TransitionFrontierSyncLedgerState::Success { .. },
                        ..
                    }
                )
            ),
            TransitionFrontierSyncAction::LedgerNextEpochPending => {
                match &state.transition_frontier.sync {
                    TransitionFrontierSyncState::Init {
                        best_tip,
                        root_block,
                        ..
                    } => SyncLedgerTarget::next_epoch(best_tip, root_block).is_some(),
                    TransitionFrontierSyncState::StakingLedgerSuccess {
                        best_tip,
                        root_block,
                        ..
                    } => SyncLedgerTarget::next_epoch(best_tip, root_block).is_some(),
                    _ => false,
                }
            }
            TransitionFrontierSyncAction::LedgerNextEpochSuccess => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::NextEpochLedgerPending(
                    TransitionFrontierSyncLedgerPending {
                        ledger: TransitionFrontierSyncLedgerState::Success { .. },
                        ..
                    }
                )
            ),
            TransitionFrontierSyncAction::LedgerRootPending => {
                match &state.transition_frontier.sync {
                    TransitionFrontierSyncState::Init {
                        best_tip,
                        root_block,
                        ..
                    }
                    | TransitionFrontierSyncState::StakingLedgerSuccess {
                        best_tip,
                        root_block,
                        ..
                    } => SyncLedgerTarget::next_epoch(best_tip, root_block).is_none(),
                    TransitionFrontierSyncState::NextEpochLedgerSuccess { .. } => true,
                    _ => false,
                }
            }
            TransitionFrontierSyncAction::LedgerRootSuccess => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::RootLedgerPending(
                    TransitionFrontierSyncLedgerPending {
                        ledger: TransitionFrontierSyncLedgerState::Success { .. },
                        ..
                    }
                )
            ),
            TransitionFrontierSyncAction::BlocksPending => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::RootLedgerSuccess { .. }
            ),
            TransitionFrontierSyncAction::BlocksPeersQuery => {
                let peers_available = state
                    .p2p
                    .ready_peers_iter()
                    .any(|(_, p)| p.channels.rpc.can_send_request());
                let sync = &state.transition_frontier.sync;
                peers_available
                    && (sync.blocks_fetch_next().is_some()
                        || sync.blocks_fetch_retry_iter().next().is_some())
            }
            TransitionFrontierSyncAction::BlocksPeerQueryInit { hash, peer_id } => {
                let check_next_hash = state
                    .transition_frontier
                    .sync
                    .blocks_fetch_next()
                    .map_or(false, |expected| &expected == hash);

                let check_peer_available = state
                    .p2p
                    .get_ready_peer(peer_id)
                    .and_then(|p| {
                        let sync_best_tip = state.transition_frontier.sync.best_tip()?;
                        let peer_best_tip = p.best_tip.as_ref()?;
                        Some(p).filter(|_| sync_best_tip.hash == peer_best_tip.hash)
                    })
                    .map_or(false, |p| p.channels.rpc.can_send_request());

                check_next_hash && check_peer_available
            }
            TransitionFrontierSyncAction::BlocksPeerQueryRetry { hash, peer_id } => {
                let check_next_hash = state
                    .transition_frontier
                    .sync
                    .blocks_fetch_retry_iter()
                    .next()
                    .map_or(false, |expected| &expected == hash);

                let check_peer_available = state
                    .p2p
                    .get_ready_peer(peer_id)
                    .and_then(|p| {
                        let sync_best_tip = state.transition_frontier.sync.best_tip()?;
                        let peer_best_tip = p.best_tip.as_ref()?;
                        Some(p).filter(|_| sync_best_tip.hash == peer_best_tip.hash)
                    })
                    .map_or(false, |p| p.channels.rpc.can_send_request());

                check_next_hash && check_peer_available
            }
            TransitionFrontierSyncAction::BlocksPeerQueryPending { hash, peer_id, .. } => state
                .transition_frontier
                .sync
                .block_state(hash)
                .map_or(false, |b| b.is_fetch_init_from_peer(peer_id)),
            TransitionFrontierSyncAction::BlocksPeerQueryError {
                peer_id, rpc_id, ..
            } => state
                .transition_frontier
                .sync
                .blocks_iter()
                .any(|s| s.is_fetch_pending_from_peer(peer_id, *rpc_id)),
            TransitionFrontierSyncAction::BlocksPeerQuerySuccess {
                peer_id,
                rpc_id,
                response,
            } => state
                .transition_frontier
                .sync
                .block_state(&response.hash)
                .filter(|s| s.is_fetch_pending_from_peer(peer_id, *rpc_id))
                .map_or(false, |s| s.block_hash() == &response.hash),
            TransitionFrontierSyncAction::BlocksFetchSuccess { hash } => state
                .transition_frontier
                .sync
                .block_state(hash)
                .map_or(false, |s| s.fetch_pending_fetched_block().is_some()),
            TransitionFrontierSyncAction::BlocksNextApplyInit => {
                state.transition_frontier.sync.blocks_apply_next().is_some()
            }
            TransitionFrontierSyncAction::BlocksNextApplyPending { hash } => state
                .transition_frontier
                .sync
                .blocks_apply_next()
                .map_or(false, |(b, _)| &b.hash == hash),
            TransitionFrontierSyncAction::BlocksNextApplySuccess { hash } => state
                .transition_frontier
                .sync
                .blocks_apply_pending()
                .map_or(false, |b| &b.hash == hash),
            TransitionFrontierSyncAction::BlocksSuccess => match &state.transition_frontier.sync {
                TransitionFrontierSyncState::BlocksPending { chain, .. } => {
                    chain.iter().all(|v| v.is_apply_success())
                }
                _ => false,
            },
            TransitionFrontierSyncAction::CommitInit => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::BlocksSuccess { .. },
            ),
            TransitionFrontierSyncAction::CommitPending => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::BlocksSuccess { .. },
            ),
            TransitionFrontierSyncAction::CommitSuccess { .. } => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::CommitPending { .. },
            ),
            TransitionFrontierSyncAction::Ledger(action) => action.is_enabled(state, time),
        }
    }
}

impl From<TransitionFrontierSyncAction> for crate::Action {
    fn from(value: TransitionFrontierSyncAction) -> Self {
        Self::TransitionFrontier(TransitionFrontierAction::Sync(value))
    }
}

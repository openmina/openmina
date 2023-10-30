use redux::ActionMeta;

use crate::consensus::consensus_effects;
use crate::event_source::event_source_effects;
use crate::external_snark_worker::{
    external_snark_worker_effects, ExternalSnarkWorkerStartAction,
    ExternalSnarkWorkerStartTimeoutAction, ExternalSnarkWorkerWorkTimeoutAction,
};
use crate::logger::logger_effects;
use crate::p2p::channels::rpc::{
    P2pChannelsRpcRequestSendAction, P2pChannelsRpcTimeoutAction, P2pRpcKind, P2pRpcRequest,
};
use crate::p2p::channels::snark::P2pChannelsSnarkRequestSendAction;
use crate::p2p::connection::incoming::P2pConnectionIncomingTimeoutAction;
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingRandomInitAction, P2pConnectionOutgoingReconnectAction,
    P2pConnectionOutgoingTimeoutAction,
};
use crate::p2p::discovery::P2pDiscoveryKademliaInitAction;
use crate::p2p::p2p_effects;
use crate::rpc::rpc_effects;
use crate::snark::snark_effects;
use crate::snark_pool::candidate::{
    SnarkPoolCandidateWorkFetchAllAction, SnarkPoolCandidateWorkVerifyNextAction,
};
use crate::snark_pool::{
    snark_pool_effects, SnarkPoolCheckTimeoutsAction, SnarkPoolP2pSendAllAction,
};
use crate::transition_frontier::sync::TransitionFrontierSyncBlocksNextApplyInitAction;
use crate::transition_frontier::transition_frontier_effects;
use crate::watched_accounts::watched_accounts_effects;
use crate::{Action, ActionWithMeta, Service, Store};

pub const MAX_PEER_PENDING_SNARKS: usize = 32;

pub fn effects<S: Service>(store: &mut Store<S>, action: ActionWithMeta) {
    store.service.recorder().action(&action);

    let (action, meta) = action.split();

    if let Some(stats) = store.service.stats() {
        stats.new_action(action.kind(), meta.clone());
    }

    logger_effects(store, meta.clone().with_action(&action));
    match action {
        // Following action gets dispatched very often, so ideally this
        // effect execution should be as light as possible.
        Action::CheckTimeouts(_) => {
            // TODO(binier): create init action and dispatch this there.
            store.dispatch(ExternalSnarkWorkerStartAction {});

            p2p_connection_timeouts(store, &meta);

            store.dispatch(P2pConnectionOutgoingRandomInitAction {});

            p2p_try_reconnect_disconnected_peers(store);

            store.dispatch(SnarkPoolCheckTimeoutsAction {});
            store.dispatch(SnarkPoolP2pSendAllAction {});

            p2p_request_best_tip_if_needed(store);

            store.dispatch(SnarkPoolCandidateWorkFetchAllAction {});
            store.dispatch(SnarkPoolCandidateWorkVerifyNextAction {});

            p2p_request_snarks_if_needed(store);

            if store.state().p2p.enough_time_elapsed(meta.time()) {
                store.dispatch(P2pDiscoveryKademliaInitAction {});
            }
            #[cfg(feature = "p2p-webrtc")]
            p2p_discovery_request(store, &meta);

            let state = store.state();
            for (peer_id, id) in state.p2p.peer_rpc_timeouts(state.time()) {
                store.dispatch(P2pChannelsRpcTimeoutAction { peer_id, id });
            }

            // TODO(binier): remove once ledger communication is async.
            store.dispatch(TransitionFrontierSyncBlocksNextApplyInitAction {});

            store.dispatch(ExternalSnarkWorkerStartTimeoutAction { now: meta.time() });
            store.dispatch(ExternalSnarkWorkerWorkTimeoutAction { now: meta.time() });
        }
        Action::EventSource(action) => {
            event_source_effects(store, meta.with_action(action));
        }
        Action::Snark(action) => {
            snark_effects(store, meta.with_action(action));
        }
        Action::Consensus(action) => {
            consensus_effects(store, meta.with_action(action));
        }
        Action::TransitionFrontier(action) => {
            transition_frontier_effects(store, meta.with_action(action));
        }
        Action::P2p(action) => {
            p2p_effects(store, meta.with_action(action));
        }
        Action::SnarkPool(action) => {
            snark_pool_effects(store, meta.with_action(action));
        }
        Action::Rpc(action) => {
            rpc_effects(store, meta.with_action(action));
        }
        Action::ExternalSnarkWorker(action) => {
            external_snark_worker_effects(store, meta.with_action(action));
        }
        Action::WatchedAccounts(action) => {
            watched_accounts_effects(store, meta.with_action(action));
        }
    }
}

fn p2p_connection_timeouts<S: Service>(store: &mut Store<S>, meta: &ActionMeta) {
    let now = meta.time();
    let p2p_connection_timeouts: Vec<_> = store
        .state()
        .p2p
        .peers
        .iter()
        .filter_map(|(peer_id, peer)| {
            let s = peer.status.as_connecting()?;
            match s.is_timed_out(now) {
                true => Some((*peer_id, s.as_outgoing().is_some())),
                false => None,
            }
        })
        .collect();

    for (peer_id, is_outgoing) in p2p_connection_timeouts {
        match is_outgoing {
            true => store.dispatch(P2pConnectionOutgoingTimeoutAction { peer_id }),
            false => store.dispatch(P2pConnectionIncomingTimeoutAction { peer_id }),
        };
    }
}

fn p2p_try_reconnect_disconnected_peers<S: Service>(store: &mut Store<S>) {
    let reconnect_actions: Vec<_> = store
        .state()
        .p2p
        .peers
        .iter()
        .filter_map(|(_, p)| p.dial_opts.clone())
        .map(|opts| P2pConnectionOutgoingReconnectAction { opts, rpc_id: None })
        .collect();
    for action in reconnect_actions {
        store.dispatch(action);
    }
}

fn p2p_request_best_tip_if_needed<S: Service>(store: &mut Store<S>) {
    // TODO(binier): refactor
    let state = store.state();
    let consensus_best_tip_hash = state.consensus.best_tip.as_ref();
    let best_tip_hash = state.transition_frontier.best_tip().map(|v| &v.hash);
    let syncing_best_tip_hash = state.transition_frontier.sync.best_tip().map(|v| &v.hash);

    if consensus_best_tip_hash.is_some()
        && consensus_best_tip_hash != best_tip_hash
        && consensus_best_tip_hash != syncing_best_tip_hash
        && state.consensus.best_tip_chain_proof.is_none()
    {
        if !state
            .p2p
            .ready_peers_iter()
            .filter_map(|(_, s)| s.channels.rpc.pending_local_rpc_kind())
            .any(|kind| matches!(kind, P2pRpcKind::BestTipWithProof))
        {
            // TODO(binier): choose randomly.
            if let Some((peer_id, id)) = state
                .p2p
                .ready_peers_iter()
                .filter(|(_, p)| p.channels.rpc.can_send_request())
                .filter(|(_, p)| {
                    p.best_tip
                        .as_ref()
                        .map_or(true, |b| Some(b.hash()) == consensus_best_tip_hash)
                })
                .map(|(peer_id, p)| (*peer_id, p.channels.rpc.next_local_rpc_id()))
                .last()
            {
                store.dispatch(P2pChannelsRpcRequestSendAction {
                    peer_id,
                    id,
                    request: P2pRpcRequest::BestTipWithProof,
                });
            }
        }
    }
}

fn p2p_request_snarks_if_needed<S: Service>(store: &mut Store<S>) {
    let state = store.state();
    let snark_reqs = state
        .p2p
        .ready_peers_iter()
        .filter(|(_, p)| p.channels.snark.can_send_request())
        .map(|(peer_id, _)| {
            let pending_snarks = state.snark_pool.candidates.peer_work_count(peer_id);
            (
                peer_id,
                MAX_PEER_PENDING_SNARKS.saturating_sub(pending_snarks),
            )
        })
        .filter(|(_, limit)| *limit > 0)
        .map(|(peer_id, limit)| (*peer_id, limit.min(u8::MAX as usize) as u8))
        .collect::<Vec<_>>();

    for (peer_id, limit) in snark_reqs {
        store.dispatch(P2pChannelsSnarkRequestSendAction { peer_id, limit });
    }
}

/// Iterate all connected peers and check the time of the last response to the peer discovery request.
/// If the elapsed time is large enough, send another discovery request.
#[cfg(feature = "p2p-webrtc")]
fn p2p_discovery_request<S: Service>(store: &mut Store<S>, meta: &ActionMeta) {
    use crate::p2p::discovery::P2pDiscoveryInitAction;

    let peer_ids = store
        .state()
        .p2p
        .ready_peers_iter()
        .filter_map(|(peer_id, status)| {
            let Some(t) = status.last_received_initial_peers else {
                return Some(*peer_id);
            };
            let elapsed = meta
                .time_as_nanos()
                .checked_sub(t.into())
                .unwrap_or_default();
            let minimal_interval = store.state().p2p.config.ask_initial_peers_interval;
            if elapsed < minimal_interval.as_nanos() as u64 {
                None
            } else {
                Some(*peer_id)
            }
        })
        .collect::<Vec<_>>();

    for peer_id in peer_ids {
        store.dispatch(P2pDiscoveryInitAction { peer_id });
    }
}

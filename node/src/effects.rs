use p2p::p2p_timeout_effects;

use crate::block_producer::{block_producer_effects, BlockProducerAction};
use crate::consensus::consensus_effects;
use crate::event_source::event_source_effects;
use crate::external_snark_worker::external_snark_worker_effects;
use crate::logger::logger_effects;
use crate::p2p::node_p2p_effects;
use crate::rpc::rpc_effects;
use crate::snark::snark_effects;
use crate::snark_pool::candidate::SnarkPoolCandidateAction;
use crate::snark_pool::{snark_pool_effects, SnarkPoolAction};
use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::transition_frontier::transition_frontier_effects;
use crate::watched_accounts::watched_accounts_effects;
use crate::{Action, ActionWithMeta, ExternalSnarkWorkerAction, Service, Store};

use crate::p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest};

#[cfg(feature = "p2p-libp2p")]
use crate::p2p::channels::rpc::P2pRpcKind;

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
            store.dispatch(ExternalSnarkWorkerAction::Start);

            p2p_timeout_effects(store, &meta);

            p2p_request_best_tip_if_needed(store);
            p2p_request_snarks_if_needed(store);

            store.dispatch(SnarkPoolAction::CheckTimeouts);
            store.dispatch(SnarkPoolAction::P2pSendAll);

            store.dispatch(SnarkPoolCandidateAction::WorkFetchAll);
            store.dispatch(SnarkPoolCandidateAction::WorkVerifyNext);

            #[cfg(feature = "p2p-webrtc")]
            p2p_discovery_request(store, &meta);

            // TODO(binier): remove once ledger communication is async.
            store.dispatch(TransitionFrontierSyncAction::BlocksNextApplyInit);

            store.dispatch(ExternalSnarkWorkerAction::StartTimeout { now: meta.time() });
            store.dispatch(ExternalSnarkWorkerAction::WorkTimeout { now: meta.time() });

            store.dispatch(BlockProducerAction::WonSlotProduceInit);
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
            node_p2p_effects(store, meta.with_action(action));
        }
        Action::SnarkPool(action) => {
            snark_pool_effects(store, meta.with_action(action));
        }
        Action::BlockProducer(action) => {
            block_producer_effects(store, meta.with_action(action));
        }
        Action::ExternalSnarkWorker(action) => {
            external_snark_worker_effects(store, meta.with_action(action));
        }
        Action::Rpc(action) => {
            rpc_effects(store, meta.with_action(action));
        }
        Action::WatchedAccounts(action) => {
            watched_accounts_effects(store, meta.with_action(action));
        }
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
        request_best_tip(store, consensus_best_tip_hash.cloned());
    }
}
use mina_p2p_messages::v2::StateHash;

#[cfg(feature = "p2p-libp2p")]
fn request_best_tip<S: Service>(store: &mut Store<S>, consensus_best_tip_hash: Option<StateHash>) {
    if !store
        .state()
        .p2p
        .ready_peers_iter()
        .filter_map(|(_, s)| s.channels.rpc.pending_local_rpc_kind())
        .any(|kind| matches!(kind, P2pRpcKind::BestTipWithProof))
    {
        // TODO(binier): choose randomly.
        if let Some((peer_id, id)) = store
            .state()
            .p2p
            .ready_peers_iter()
            .filter(|(_, p)| p.channels.rpc.can_send_request())
            .filter(|(_, p)| {
                p.best_tip
                    .as_ref()
                    .map_or(true, |b| Some(b.hash()) == consensus_best_tip_hash.as_ref())
            })
            .map(|(peer_id, p)| (*peer_id, p.channels.rpc.next_local_rpc_id()))
            .last()
        {
            store.dispatch(P2pChannelsRpcAction::RequestSend {
                peer_id,
                id,
                request: P2pRpcRequest::BestTipWithProof,
            });
        }
    }
}

#[cfg(not(feature = "p2p-libp2p"))]
fn request_best_tip<S: Service>(store: &mut Store<S>, _consensus_best_tip_hash: Option<StateHash>) {
    if let Some((peer_id, streams)) = store
        .state()
        .p2p
        .network
        .scheduler
        .rpc_outgoing_streams
        .iter()
        .last()
    {
        if let Some((_, state)) = streams.iter().last() {
            store.dispatch(P2pChannelsRpcAction::RequestSend {
                peer_id: *peer_id,
                id: state.last_id as _,
                request: P2pRpcRequest::BestTipWithProof,
            });
        }
    }
}

fn p2p_request_snarks_if_needed<S: Service>(store: &mut Store<S>) {
    use p2p::channels::snark::P2pChannelsSnarkAction;

    const MAX_PEER_PENDING_SNARKS: usize = 32;

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
        store.dispatch(P2pChannelsSnarkAction::RequestSend { peer_id, limit });
    }
}

/// Iterate all connected peers and check the time of the last response to the peer discovery request.
/// If the elapsed time is large enough, send another discovery request.
#[cfg(feature = "p2p-webrtc")]
fn p2p_discovery_request<S: Service>(store: &mut Store<S>, meta: &redux::ActionMeta) {
    use crate::p2p::discovery::P2pDiscoveryAction;

    let peer_ids = store
        .state()
        .p2p
        .ready_peers_iter()
        .filter_map(|(peer_id, _)| {
            let Some(t) = store
                .state()
                .p2p
                .kademlia
                .peer_timestamp
                .get(peer_id)
                .cloned()
            else {
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
        store.dispatch(P2pDiscoveryAction::Init { peer_id });
    }
}

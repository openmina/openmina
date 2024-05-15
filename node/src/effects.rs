use openmina_core::log::system_time;
use p2p::p2p_timeout_effects;

use crate::block_producer::{block_producer_effects, BlockProducerAction};
use crate::event_source::event_source_effects;
use crate::external_snark_worker::external_snark_worker_effects;
use crate::ledger::ledger_effects;
use crate::ledger::read::LedgerReadAction;
use crate::logger::logger_effects;
use crate::p2p::node_p2p_effects;
use crate::rpc::rpc_effects;
use crate::snark::snark_effects;
use crate::snark_pool::candidate::SnarkPoolCandidateAction;
use crate::snark_pool::{snark_pool_effects, SnarkPoolAction};
use crate::transition_frontier::genesis::TransitionFrontierGenesisAction;
use crate::transition_frontier::transition_frontier_effects;
use crate::{p2p_ready, Action, ActionWithMeta, ExternalSnarkWorkerAction, Service, Store};

use crate::p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest};

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
            // TODO(binier): create init action and dispatch these there.
            store.dispatch(TransitionFrontierGenesisAction::LedgerLoadInit);
            store.dispatch(ExternalSnarkWorkerAction::Start);

            if store.state().p2p.ready().is_some() {
                p2p_timeout_effects(store, &meta);

                p2p_request_best_tip_if_needed(store);
                p2p_request_snarks_if_needed(store);
            }

            store.dispatch(SnarkPoolAction::CheckTimeouts);
            store.dispatch(SnarkPoolAction::P2pSendAll);

            store.dispatch(SnarkPoolCandidateAction::WorkFetchAll);
            store.dispatch(SnarkPoolCandidateAction::WorkVerifyNext);

            store.dispatch(ExternalSnarkWorkerAction::StartTimeout { now: meta.time() });
            store.dispatch(ExternalSnarkWorkerAction::WorkTimeout { now: meta.time() });

            store.dispatch(BlockProducerAction::WonSlotProduceInit);
            store.dispatch(BlockProducerAction::BlockInject);
            store.dispatch(LedgerReadAction::FindTodos);
        }
        Action::EventSource(action) => {
            event_source_effects(store, meta.with_action(action));
        }
        Action::Snark(action) => {
            snark_effects(store, meta.with_action(action));
        }
        Action::Consensus(_) => {
            // Handled by reducer
        }
        Action::TransactionPool(_action) => {}
        Action::TransactionPoolEffect(action) => {
            action.effects(store);
        }
        Action::TransitionFrontier(action) => {
            transition_frontier_effects(store, meta.with_action(action));
        }
        Action::P2p(action) => {
            node_p2p_effects(store, meta.with_action(action));
        }
        Action::Ledger(action) => {
            ledger_effects(store, meta.with_action(action));
        }
        Action::SnarkPool(_) => {}
        Action::SnarkPoolEffect(action) => {
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
        Action::WatchedAccounts(_) => {
            // Handled by reducer
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

fn request_best_tip<S: Service>(store: &mut Store<S>, _consensus_best_tip_hash: Option<StateHash>) {
    let p2p = p2p_ready!(store.state().p2p, "request_best_tip", system_time());
    // TODO: choose peer that has this channel capability
    if let Some((peer_id, streams)) = p2p.network.scheduler.rpc_outgoing_streams.iter().last() {
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
    let p2p = p2p_ready!(state.p2p, "p2p_request_snarks_if_needed", system_time());
    let snark_reqs = p2p
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

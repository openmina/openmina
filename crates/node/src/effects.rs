use mina_core::log::system_time;
use rand::prelude::*;

use crate::{
    block_producer::BlockProducerAction,
    block_producer_effectful::block_producer_effects,
    event_source::event_source_effects,
    external_snark_worker_effectful::external_snark_worker_effectful_effects,
    ledger::read::LedgerReadAction,
    ledger_effectful::ledger_effectful_effects,
    logger::logger_effects,
    p2p::node_p2p_effects,
    p2p_ready,
    rpc_effectful::rpc_effects,
    snark::snark_effects,
    snark_pool::{candidate::SnarkPoolCandidateAction, snark_pool_effects, SnarkPoolAction},
    transaction_pool::candidate::TransactionPoolCandidateAction,
    transition_frontier::{genesis::TransitionFrontierGenesisAction, transition_frontier_effects},
    Action, ActionWithMeta, ExternalSnarkWorkerAction, Service, Store, TransactionPoolAction,
};

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

            store.dispatch(TransitionFrontierGenesisAction::ProveInit);

            if store.state().p2p.ready().is_some() {
                p2p_request_best_tip_if_needed(store);
                p2p_request_transactions_if_needed(store);
                p2p_request_snarks_if_needed(store);
            }

            store.dispatch(TransactionPoolAction::P2pSendAll);
            store.dispatch(TransactionPoolCandidateAction::FetchAll);
            store.dispatch(TransactionPoolCandidateAction::VerifyNext);

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
        Action::TransactionPoolEffect(action) => {
            action.effects(store);
        }
        Action::TransitionFrontier(action) => {
            transition_frontier_effects(store, meta.with_action(action));
        }
        Action::P2pEffectful(action) => {
            node_p2p_effects(store, meta.with_action(action));
        }
        Action::LedgerEffects(action) => {
            ledger_effectful_effects(store, meta.with_action(action));
        }
        Action::SnarkPoolEffect(action) => {
            snark_pool_effects(store, meta.with_action(action));
        }
        Action::BlockProducerEffectful(action) => {
            block_producer_effects(store, meta.with_action(action));
        }
        Action::ExternalSnarkWorkerEffects(action) => {
            external_snark_worker_effectful_effects(store, meta.with_action(action));
        }
        Action::RpcEffectful(action) => {
            rpc_effects(store, meta.with_action(action));
        }
        Action::BlockProducer(_)
        | Action::SnarkPool(_)
        | Action::ExternalSnarkWorker(_)
        | Action::TransactionPool(_)
        | Action::Ledger(_)
        | Action::Rpc(_)
        | Action::WatchedAccounts(_)
        | Action::P2pCallbacks(_)
        | Action::P2p(_) => {
            // Handled by reducer
        }
    }
}

fn p2p_request_best_tip_if_needed<S: Service>(store: &mut Store<S>) {
    // TODO(binier): refactor
    let state = store.state();
    let best_candidate = state.transition_frontier.candidates.best_verified();
    let best_candidate_hash = best_candidate.map(|s| s.block.hash());
    let best_tip_hash = state.transition_frontier.best_tip().map(|v| &v.hash);
    let syncing_best_tip_hash = state.transition_frontier.sync.best_tip().map(|v| &v.hash);

    if best_candidate.is_some()
        && best_candidate_hash != best_tip_hash
        && best_candidate_hash != syncing_best_tip_hash
        && best_candidate.is_some_and(|s| s.chain_proof.is_none())
    {
        request_best_tip(store, best_candidate_hash.cloned());
    }
}
use mina_p2p_messages::v2::StateHash;

fn request_best_tip<S: Service>(store: &mut Store<S>, consensus_best_tip_hash: Option<StateHash>) {
    let p2p = p2p_ready!(store.state().p2p, "request_best_tip", system_time());

    let (ready_peers, ready_peers_matching_best_tip) = p2p.ready_rpc_peers_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut all, mut matching), (peer_id, peer)| {
            let rpc_id = peer.channels.next_local_rpc_id();
            if peer.best_tip.as_ref().map(|b| b.hash()) == consensus_best_tip_hash.as_ref() {
                matching.push((*peer_id, rpc_id));
            } else if matching.is_empty() {
                all.push((*peer_id, rpc_id));
            }
            (all, matching)
        },
    );

    let peers = if !ready_peers_matching_best_tip.is_empty() {
        ready_peers_matching_best_tip
    } else {
        ready_peers
    };

    if let Some((peer_id, id)) = peers.choose(&mut store.state().pseudo_rng()) {
        store.dispatch(P2pChannelsRpcAction::RequestSend {
            peer_id: *peer_id,
            id: *id,
            request: Box::new(P2pRpcRequest::BestTipWithProof),
            on_init: None,
        });
    }
}

fn p2p_request_transactions_if_needed<S: Service>(store: &mut Store<S>) {
    use p2p::channels::transaction::P2pChannelsTransactionAction;

    const MAX_PEER_PENDING_TXS: usize = 32;

    let state = store.state();
    let p2p = p2p_ready!(
        state.p2p,
        "p2p_request_transactions_if_needed",
        system_time()
    );
    let transaction_reqs = p2p
        .ready_peers_iter()
        .filter(|(_, p)| p.channels.transaction.can_send_request())
        .map(|(peer_id, _)| {
            let pending_txs = state
                .transaction_pool
                .candidates
                .peer_transaction_count(peer_id);
            (peer_id, MAX_PEER_PENDING_TXS.saturating_sub(pending_txs))
        })
        .filter(|(_, limit)| *limit > 0)
        .map(|(peer_id, limit)| (*peer_id, limit.min(u8::MAX as usize) as u8))
        .collect::<Vec<_>>();

    for (peer_id, limit) in transaction_reqs {
        store.dispatch(P2pChannelsTransactionAction::RequestSend { peer_id, limit });
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

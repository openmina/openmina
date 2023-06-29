use crate::consensus::consensus_effects;
use crate::event_source::event_source_effects;
use crate::job_commitment::{
    job_commitment_effects, JobCommitmentCheckTimeoutsAction, JobCommitmentP2pSendAllAction,
};
use crate::ledger::ledger_effects;
use crate::logger::logger_effects;
use crate::p2p::channels::rpc::P2pChannelsRpcTimeoutAction;
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingRandomInitAction, P2pConnectionOutgoingReconnectAction,
};
use crate::p2p::p2p_effects;
use crate::rpc::rpc_effects;
use crate::snark::snark_effects;
use crate::transition_frontier::transition_frontier_effects;
use crate::watched_accounts::watched_accounts_effects;
use crate::{Action, ActionWithMeta, Service, Store};

pub fn effects<S: Service>(store: &mut Store<S>, action: ActionWithMeta) {
    let (action, meta) = action.split();

    if let Some(stats) = store.service.stats() {
        stats.new_action(action.kind(), meta.clone());
    }

    logger_effects(store, meta.clone().with_action(&action));
    match action {
        Action::CheckTimeouts(_) => {
            store.dispatch(P2pConnectionOutgoingRandomInitAction {});

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

            store.dispatch(JobCommitmentCheckTimeoutsAction {});
            store.dispatch(JobCommitmentP2pSendAllAction {});

            let state = store.state();
            for (peer_id, id) in state.p2p.peer_rpc_timeouts(state.time()) {
                store.dispatch(P2pChannelsRpcTimeoutAction { peer_id, id });
            }
        }
        Action::EventSource(action) => {
            event_source_effects(store, meta.with_action(action));
        }
        Action::Ledger(action) => {
            ledger_effects(store, meta.with_action(action));
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
        Action::JobCommitment(action) => {
            job_commitment_effects(store, meta.with_action(action));
        }
        Action::Rpc(action) => {
            rpc_effects(store, meta.with_action(action));
        }
        Action::WatchedAccounts(action) => {
            watched_accounts_effects(store, meta.with_action(action));
        }
    }
}

use crate::consensus::consensus_effects;
use crate::event_source::event_source_effects;
use crate::logger::logger_effects;
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingReconnectAction,
};
use crate::p2p::p2p_effects;
use crate::p2p::rpc::outgoing::{P2pRpcOutgoingInitAction, P2pRpcRequestor};
use crate::p2p::rpc::P2pRpcRequest;
use crate::rpc::rpc_effects;
use crate::snark::snark_effects;
use crate::watched_accounts::{
    watched_accounts_effects, WatchedAccountLedgerInitialState,
    WatchedAccountsLedgerInitialStateGetRetryAction,
};
use crate::{Action, ActionWithMeta, Service, Store};

pub fn effects<S: Service>(store: &mut Store<S>, action: ActionWithMeta) {
    let (action, meta) = action.split();

    if let Some(stats) = store.service.stats() {
        stats.new_action(action.kind(), meta.clone());
    }

    logger_effects(store, meta.clone().with_action(&action));
    match action {
        Action::CheckTimeouts(_) => {
            for peer_id in store.state().p2p.ready_peers() {
                store.dispatch(P2pRpcOutgoingInitAction {
                    peer_id,
                    rpc_id: match store.state().p2p.get_ready_peer(&peer_id) {
                        Some(p) => p.rpc.outgoing.next_req_id(),
                        None => return,
                    },
                    request: P2pRpcRequest::BestTipGet(()),
                    requestor: P2pRpcRequestor::Interval,
                });
            }

            let reconnect_actions: Vec<_> = store
                .state()
                .p2p
                .peers
                .iter()
                .map(|(id, p)| P2pConnectionOutgoingReconnectAction {
                    opts: P2pConnectionOutgoingInitOpts {
                        peer_id: id.clone(),
                        addrs: p.dial_addrs.clone(),
                    },
                    rpc_id: None,
                })
                .collect();
            for action in reconnect_actions {
                store.dispatch(action);
            }

            let actions = store
                .state()
                .watched_accounts
                .iter()
                .filter(|(_, a)| !a.initial_state.is_success())
                .map(
                    |(pub_key, _)| WatchedAccountsLedgerInitialStateGetRetryAction {
                        pub_key: pub_key.clone(),
                    },
                )
                .collect::<Vec<_>>();
            for action in actions {
                store.dispatch(action);
            }
        }
        Action::EventSource(action) => {
            event_source_effects(store, meta.with_action(action));
        }
        Action::P2p(action) => {
            p2p_effects(store, meta.with_action(action));
        }
        Action::Snark(action) => {
            snark_effects(store, meta.with_action(action));
        }
        Action::Consensus(action) => {
            consensus_effects(store, meta.with_action(action));
        }
        Action::Rpc(action) => {
            rpc_effects(store, meta.with_action(action));
        }
        Action::WatchedAccounts(action) => {
            watched_accounts_effects(store, meta.with_action(action));
        }
    }
}

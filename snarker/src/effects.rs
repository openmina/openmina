use crate::event_source::event_source_effects;
use crate::job_commitment::{
    job_commitment_effects, JobCommitmentCheckTimeoutsAction, JobCommitmentP2pSendAllAction,
};
use crate::logger::logger_effects;
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingRandomInitAction, P2pConnectionOutgoingReconnectAction,
};
use crate::p2p::p2p_effects;
use crate::rpc::rpc_effects;
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

            // for peer_id in store.state().p2p.ready_peers() {
            // store.dispatch(P2pRpcOutgoingInitAction {
            //     peer_id,
            //     rpc_id: match store.state().p2p.get_ready_peer(&peer_id) {
            //         Some(p) => p.rpc.outgoing.next_req_id(),
            //         None => return,
            //     },
            //     request: P2pRpcRequest::BestTipGet(()),
            //     requestor: P2pRpcRequestor::Interval,
            // });
            // }

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
        }
        Action::EventSource(action) => {
            event_source_effects(store, meta.with_action(action));
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
    }
}

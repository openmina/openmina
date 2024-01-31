use std::collections::BTreeMap;

use snark::work_verify::SnarkWorkVerifyAction;

use crate::p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest};
use crate::p2p::disconnection::{P2pDisconnectionInitAction, P2pDisconnectionReason};
use crate::Store;

use super::{SnarkPoolCandidateAction, SnarkPoolCandidateActionWithMeta};

pub fn snark_pool_candidate_effects<S: redux::Service>(
    store: &mut Store<S>,
    action: SnarkPoolCandidateActionWithMeta,
) {
    let (action, _) = action.split();
    match action {
        SnarkPoolCandidateAction::InfoReceived { .. } => {}
        SnarkPoolCandidateAction::WorkFetchAll => {
            let state = store.state();
            let peers = state.p2p.ready_peers_iter().map(|(id, _)| *id);
            let get_order = |job_id: &_| {
                state
                    .snark_pool
                    .get(job_id)
                    .map(|job| job.order)
                    .unwrap_or(usize::MAX)
            };
            let list = state
                .snark_pool
                .candidates
                .peers_next_work_to_fetch(peers, get_order);

            for (peer_id, job_id) in list {
                store.dispatch(SnarkPoolCandidateAction::WorkFetchInit { peer_id, job_id });
            }
        }
        SnarkPoolCandidateAction::WorkFetchInit { peer_id, job_id } => {
            let Some(peer) = store.state().p2p.get_ready_peer(&peer_id) else {
                return;
            };
            let rpc_id = peer.channels.rpc.next_local_rpc_id();
            store.dispatch(P2pChannelsRpcAction::RequestSend {
                peer_id,
                id: rpc_id,
                request: P2pRpcRequest::Snark(job_id.clone()),
            });
            store.dispatch(SnarkPoolCandidateAction::WorkFetchPending {
                peer_id,
                job_id: job_id.clone(),
                rpc_id,
            });
        }
        SnarkPoolCandidateAction::WorkFetchPending { .. } => {}
        SnarkPoolCandidateAction::WorkReceived { .. } => {}
        SnarkPoolCandidateAction::WorkVerifyNext => {
            let state = store.state();
            let job_id_orders = state
                .snark_pool
                .range(..)
                .map(|(_, v)| (v.order, &v.id))
                .collect::<BTreeMap<_, _>>();
            let job_ids_ordered_iter = job_id_orders.into_iter().map(|(_, id)| id);
            let batch = state
                .snark_pool
                .candidates
                .get_batch_to_verify(job_ids_ordered_iter);
            let Some((peer_id, batch)) = batch else {
                return;
            };

            let req_id = state.snark.work_verify.next_req_id();
            let job_ids = batch.iter().map(|v| v.job_id()).collect::<Vec<_>>();
            let sender = peer_id.to_string();
            store.dispatch(SnarkWorkVerifyAction::Init {
                req_id,
                batch,
                sender,
            });
            store.dispatch(SnarkPoolCandidateAction::WorkVerifyPending {
                peer_id,
                job_ids,
                verify_id: req_id,
            });
        }
        SnarkPoolCandidateAction::WorkVerifyPending { .. } => {}
        SnarkPoolCandidateAction::WorkVerifyError { peer_id, .. } => {
            // TODO(binier): blacklist peer
            store.dispatch(P2pDisconnectionInitAction {
                peer_id,
                reason: P2pDisconnectionReason::SnarkPoolVerifyError,
            });
        }
        SnarkPoolCandidateAction::WorkVerifySuccess { .. } => {
            // action for adding verified snarks to snark pool is called
            // in snark/work_verify effects. That is by design as we might
            // remove work pending verification if we receive better snark
            // from same peer. But since we have already started verification,
            // we might as well use it's result.
        }
        SnarkPoolCandidateAction::PeerPrune { .. } => {}
    }
}

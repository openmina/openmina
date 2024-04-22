use super::{SnarkPoolCandidateAction, SnarkPoolCandidateActionWithMeta};
use crate::p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest};
use crate::p2p::disconnection::{P2pDisconnectionAction, P2pDisconnectionReason};
use crate::{Action, SnarkPoolAction, Store};
use snark::work_verify::SnarkWorkVerifyAction;
use std::collections::BTreeMap;

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
                verify_success_cb: redux::Callback::new(|args| {
                    let (peer_id, verify_id, batch) = *args.downcast().expect("correct arguments");
                    Box::<Action>::new(
                        SnarkPoolCandidateAction::WorkVerifySuccess {
                            peer_id,
                            verify_id,
                            batch,
                        }
                        .into(),
                    )
                }),
                verify_error_cb: redux::Callback::new(|args| {
                    let (peer_id, verify_id) = *args.downcast().expect("correct arguments");
                    Box::<Action>::new(
                        SnarkPoolCandidateAction::WorkVerifyError { peer_id, verify_id }.into(),
                    )
                }),
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
            store.dispatch(P2pDisconnectionAction::Init {
                peer_id,
                reason: P2pDisconnectionReason::SnarkPoolVerifyError,
            });
        }
        SnarkPoolCandidateAction::WorkVerifySuccess { peer_id, batch, .. } => {
            for snark in batch {
                store.dispatch(SnarkPoolAction::WorkAdd {
                    snark,
                    sender: peer_id,
                });
            }
        }
        SnarkPoolCandidateAction::PeerPrune { .. } => {}
    }
}

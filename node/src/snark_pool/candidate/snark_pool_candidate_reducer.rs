use std::collections::BTreeMap;

use crate::{p2p_ready, SnarkPoolAction};
use openmina_core::snark::{Snark, SnarkJobId};
use p2p::{
    channels::rpc::{P2pChannelsRpcAction, P2pRpcId, P2pRpcRequest},
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    BroadcastMessageId, P2pNetworkPubsubAction, PeerId,
};
use snark::{work_verify::SnarkWorkVerifyAction, work_verify_effectful::SnarkWorkVerifyId};

use super::{
    SnarkPoolCandidateAction, SnarkPoolCandidateActionWithMetaRef, SnarkPoolCandidatesState,
};

impl SnarkPoolCandidatesState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: SnarkPoolCandidateActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            SnarkPoolCandidateAction::InfoReceived { peer_id, info } => {
                state.info_received(meta.time(), *peer_id, info.clone());
            }
            SnarkPoolCandidateAction::WorkFetchAll => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let p2p = p2p_ready!(global_state.p2p, meta.time());
                let peers = p2p.ready_peers_iter().map(|(id, _)| *id);
                let get_order = |job_id: &_| {
                    global_state
                        .snark_pool
                        .get(job_id)
                        .map(|job| job.order)
                        .unwrap_or(usize::MAX)
                };
                let list = global_state
                    .snark_pool
                    .candidates
                    .peers_next_work_to_fetch(peers, get_order);

                for (peer_id, job_id) in list {
                    dispatcher.push(SnarkPoolCandidateAction::WorkFetchInit { peer_id, job_id });
                }
            }
            SnarkPoolCandidateAction::WorkFetchInit { peer_id, job_id } => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let peer_id = *peer_id;
                let job_id = job_id.clone();
                let p2p = p2p_ready!(global_state.p2p, meta.time());
                let Some(peer) = p2p.get_ready_peer(&peer_id) else {
                    return;
                };
                let rpc_id = peer.channels.next_local_rpc_id();

                dispatcher.push(P2pChannelsRpcAction::RequestSend {
                    peer_id,
                    id: rpc_id,
                    request: Box::new(P2pRpcRequest::Snark(job_id.clone())),
                    on_init: Some(redux::callback!(
                        on_send_p2p_snark_rpc_request(
                            (peer_id: PeerId, rpc_id: P2pRpcId, request: P2pRpcRequest)
                        ) -> crate::Action {
                            let P2pRpcRequest::Snark(job_id) = request else {
                                unreachable!()
                            };
                            SnarkPoolCandidateAction::WorkFetchPending {
                                job_id,
                                peer_id,
                                rpc_id,
                            }
                        }
                    )),
                });
            }
            SnarkPoolCandidateAction::WorkFetchPending {
                peer_id,
                job_id,
                rpc_id,
            } => {
                state.work_fetch_pending(meta.time(), peer_id, job_id, *rpc_id);
            }
            SnarkPoolCandidateAction::WorkFetchError { peer_id, job_id } => {
                state.peer_work_remove(*peer_id, job_id);
            }
            SnarkPoolCandidateAction::WorkFetchSuccess { peer_id, work } => {
                state.work_received(meta.time(), *peer_id, work.clone());
            }
            SnarkPoolCandidateAction::WorkVerifyNext => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();

                let job_id_orders = global_state
                    .snark_pool
                    .range(..)
                    .map(|(_, v)| (v.order, &v.id))
                    .collect::<BTreeMap<_, _>>();
                let job_ids_ordered_iter = job_id_orders.into_values();
                let batch = global_state
                    .snark_pool
                    .candidates
                    .get_batch_to_verify(job_ids_ordered_iter);
                let Some((peer_id, batch)) = batch else {
                    return;
                };

                let req_id = global_state.snark.work_verify.next_req_id();
                let job_ids = batch.iter().map(|v| v.job_id()).collect::<Vec<_>>();
                let sender = peer_id.to_string();
                dispatcher.push(SnarkWorkVerifyAction::Init {
                    req_id,
                    batch,
                    sender,
                    on_success: redux::callback!(
                        on_snark_pool_candidate_work_verify_success((req_id: SnarkWorkVerifyId, sender: String, batch: Vec<Snark>)) -> crate::Action {
                            SnarkPoolCandidateAction::WorkVerifySuccess {
                                peer_id: sender.parse().unwrap(),
                                verify_id: req_id,
                                batch
                            }
                        }),
                    on_error: redux::callback!(
                        on_snark_pool_candidate_work_verify_error((req_id: SnarkWorkVerifyId, sender: String, batch: Vec<SnarkJobId>)) -> crate::Action {
                            SnarkPoolCandidateAction::WorkVerifyError {
                                peer_id: sender.parse().unwrap(),
                                verify_id: req_id,
                                batch
                            }
                        }),
                });
                dispatcher.push(SnarkPoolCandidateAction::WorkVerifyPending {
                    peer_id,
                    job_ids,
                    verify_id: req_id,
                });
            }
            SnarkPoolCandidateAction::WorkVerifyPending {
                peer_id,
                job_ids,
                verify_id,
            } => {
                state.verify_pending(meta.time(), peer_id, *verify_id, job_ids);
            }
            SnarkPoolCandidateAction::WorkVerifyError {
                peer_id,
                verify_id,
                batch,
            } => {
                state.verify_result(meta.time(), peer_id, *verify_id, Err(()));

                // TODO(binier): blacklist peer
                let dispatcher = state_context.into_dispatcher();
                let peer_id = *peer_id;
                dispatcher.push(P2pDisconnectionAction::Init {
                    peer_id,
                    reason: P2pDisconnectionReason::SnarkPoolVerifyError,
                });

                // TODO: This is not correct. We are rejecting all snark messages, but the fact that the batch
                // failed to verify means that there is at least one invalid snark in the batch, not that all of them
                // are invalid.
                // Instead, what should happen here is that we split the batch in two and try to verify the two batches
                // again. Repeating until batches don't fail to verify anymore, or each batch is of size 1.
                // It may also be worth capping the batch sizes to 10.
                for snark_job_id in batch {
                    dispatcher.push(P2pNetworkPubsubAction::RejectMessage {
                        message_id: Some(BroadcastMessageId::Snark {
                            job_id: snark_job_id.clone(),
                        }),
                        peer_id: None,
                        reason: "Snark work verification failed".to_string(),
                    });
                }
            }
            SnarkPoolCandidateAction::WorkVerifySuccess {
                peer_id,
                verify_id,
                batch,
            } => {
                state.verify_result(meta.time(), peer_id, *verify_id, Ok(()));

                // Dispatch
                let dispatcher = state_context.into_dispatcher();

                for snark in batch {
                    dispatcher.push(SnarkPoolAction::WorkAdd {
                        snark: snark.clone(),
                        sender: *peer_id,
                        is_sender_local: false,
                    });
                }
            }
            SnarkPoolCandidateAction::PeerPrune { peer_id } => {
                state.peer_remove(*peer_id);
            }
        }
    }
}

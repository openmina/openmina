use std::collections::BTreeMap;

use p2p::{
    channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest},
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
};
use snark::work_verify::SnarkWorkVerifyAction;

use super::{
    SnarkPoolCandidateAction, SnarkPoolCandidateActionWithMetaRef, SnarkPoolCandidatesState,
};

impl SnarkPoolCandidatesState {
    pub fn reducer(
        &mut self,
        action: SnarkPoolCandidateActionWithMetaRef<'_>,
        global_state: &crate::State,
        dispatcher: &mut redux::ActionQueue<crate::Action, crate::State>,
    ) {
        let (action, meta) = action.split();
        match action {
            SnarkPoolCandidateAction::InfoReceived { peer_id, info } => {
                self.info_received(meta.time(), *peer_id, info.clone());
            }
            SnarkPoolCandidateAction::WorkFetchAll => {
                let peers = global_state.p2p.ready_peers_iter().map(|(id, _)| *id);
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
                let Some(peer) = global_state.p2p.get_ready_peer(&peer_id) else {
                    return;
                };
                let rpc_id = peer.channels.rpc.next_local_rpc_id();
                let peer_id = *peer_id;
                dispatcher.push(P2pChannelsRpcAction::RequestSend {
                    peer_id,
                    id: rpc_id,
                    request: P2pRpcRequest::Snark(job_id.clone()),
                });
                dispatcher.push(SnarkPoolCandidateAction::WorkFetchPending {
                    peer_id,
                    job_id: job_id.clone(),
                    rpc_id,
                });
            }
            SnarkPoolCandidateAction::WorkFetchPending {
                peer_id,
                job_id,
                rpc_id,
            } => {
                self.work_fetch_pending(meta.time(), peer_id, job_id, *rpc_id);
            }
            SnarkPoolCandidateAction::WorkReceived { peer_id, work } => {
                self.work_received(meta.time(), *peer_id, work.clone());
            }
            SnarkPoolCandidateAction::WorkVerifyNext => {
                let job_id_orders = global_state
                    .snark_pool
                    .range(..)
                    .map(|(_, v)| (v.order, &v.id))
                    .collect::<BTreeMap<_, _>>();
                let job_ids_ordered_iter = job_id_orders.into_iter().map(|(_, id)| id);
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
                self.verify_pending(meta.time(), peer_id, *verify_id, job_ids);
            }
            SnarkPoolCandidateAction::WorkVerifyError { peer_id, verify_id } => {
                self.verify_result(meta.time(), peer_id, *verify_id, Err(()));

                // TODO(binier): blacklist peer
                dispatcher.push(P2pDisconnectionAction::Init {
                    peer_id: *peer_id,
                    reason: P2pDisconnectionReason::SnarkPoolVerifyError,
                });
            }
            SnarkPoolCandidateAction::WorkVerifySuccess { peer_id, verify_id } => {
                self.verify_result(meta.time(), peer_id, *verify_id, Ok(()));

                // action for adding verified snarks to snark pool is called
                // in snark/work_verify effects. That is by design as we might
                // remove work pending verification if we receive better snark
                // from same peer. But since we have already started verification,
                // we might as well use it's result.
            }
            SnarkPoolCandidateAction::PeerPrune { peer_id } => {
                self.peer_remove(*peer_id);
            }
        }
    }
}

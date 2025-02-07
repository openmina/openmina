#![allow(clippy::unit_arg)]

use crate::{p2p_ready, TransactionPoolAction};
use openmina_core::transaction::TransactionPoolMessageSource;
use p2p::{
    channels::rpc::{P2pChannelsRpcAction, P2pRpcId, P2pRpcRequest},
    PeerId,
};

use super::{
    TransactionPoolCandidateAction, TransactionPoolCandidateActionWithMetaRef,
    TransactionPoolCandidatesState,
};

impl TransactionPoolCandidatesState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: TransactionPoolCandidateActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransactionPoolCandidateAction::InfoReceived { peer_id, info } => {
                state.info_received(meta.time(), *peer_id, info.clone());
            }
            TransactionPoolCandidateAction::FetchAll => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let p2p = p2p_ready!(global_state.p2p, meta.time());
                let peers = p2p.ready_peers_iter().map(|(id, _)| *id);
                let get_order = |_hash: &_| {
                    // TODO(binier)
                    0
                };
                let list = global_state
                    .transaction_pool
                    .candidates
                    .peers_next_transactions_to_fetch(peers, get_order);

                for (peer_id, hash) in list {
                    dispatcher.push(TransactionPoolCandidateAction::FetchInit { peer_id, hash });
                }
            }
            TransactionPoolCandidateAction::FetchInit { peer_id, hash } => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let peer_id = *peer_id;
                let hash = hash.clone();
                let p2p = p2p_ready!(global_state.p2p, meta.time());
                let Some(peer) = p2p.get_ready_peer(&peer_id) else {
                    return;
                };
                let rpc_id = peer.channels.next_local_rpc_id();

                dispatcher.push(P2pChannelsRpcAction::RequestSend {
                    peer_id,
                    id: rpc_id,
                    request: Box::new(P2pRpcRequest::Transaction(hash.clone())),
                    on_init: Some(redux::callback!(
                        on_send_p2p_snark_rpc_request(
                            (peer_id: PeerId, rpc_id: P2pRpcId, request: P2pRpcRequest)
                        ) -> crate::Action {
                            let P2pRpcRequest::Transaction(hash) = request else {
                                unreachable!()
                            };
                            TransactionPoolCandidateAction::FetchPending {
                                hash,
                                peer_id,
                                rpc_id,
                            }
                        }
                    )),
                });
            }
            TransactionPoolCandidateAction::FetchPending {
                peer_id,
                hash,
                rpc_id,
            } => {
                state.fetch_pending(meta.time(), peer_id, hash, *rpc_id);
            }
            TransactionPoolCandidateAction::FetchError { peer_id, hash } => {
                state.peer_transaction_remove(*peer_id, hash);
            }
            TransactionPoolCandidateAction::FetchSuccess {
                peer_id,
                transaction,
            } => {
                state.transaction_received(meta.time(), *peer_id, transaction.clone());
            }
            TransactionPoolCandidateAction::VerifyNext => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();

                let batch = global_state
                    .transaction_pool
                    .candidates
                    .get_batch_to_verify();
                let Some((peer_id, batch)) = batch else {
                    return;
                };

                let transaction_hashes = batch.iter().map(|tx| tx.hash().clone()).collect();
                dispatcher.push(TransactionPoolAction::StartVerify {
                    commands: batch.into_iter().collect(),
                    from_source: TransactionPoolMessageSource::None,
                });
                dispatcher.push(TransactionPoolCandidateAction::VerifyPending {
                    peer_id,
                    transaction_hashes,
                    verify_id: (),
                });
            }
            TransactionPoolCandidateAction::VerifyPending {
                peer_id,
                transaction_hashes,
                verify_id,
            } => {
                state.verify_pending(meta.time(), peer_id, *verify_id, transaction_hashes);
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransactionPoolCandidateAction::VerifySuccess {
                    peer_id: *peer_id,
                    verify_id: *verify_id,
                });
            }
            TransactionPoolCandidateAction::VerifyError {
                peer_id: _,
                verify_id: _,
            } => {
                unreachable!("TODO(binier)");
                // state.verify_result(meta.time(), peer_id, *verify_id, Err(()));

                // // TODO(binier): blacklist peer
                // let dispatcher = state_context.into_dispatcher();
                // let peer_id = *peer_id;
                // dispatcher.push(P2pDisconnectionAction::Init {
                //     peer_id,
                //     reason: P2pDisconnectionReason::TransactionPoolVerifyError,
                // });
            }
            TransactionPoolCandidateAction::VerifySuccess { peer_id, verify_id } => {
                state.verify_result(meta.time(), peer_id, *verify_id, Ok(()));
            }
            TransactionPoolCandidateAction::PeerPrune { peer_id } => {
                state.peer_remove(*peer_id);
            }
        }
    }
}

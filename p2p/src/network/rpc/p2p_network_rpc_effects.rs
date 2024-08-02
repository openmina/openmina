use std::sync::Arc;

use mina_p2p_messages::{
    rpc,
    rpc_kernel::{
        PayloadBinprotReader, QueryHeader, ResponseHeader, RpcMethod, RpcQueryReadError,
        RpcResponseReadError,
    },
    v2,
    versioned::Ver,
};
use openmina_core::{error, fuzz_maybe, fuzzed_maybe};

use crate::{
    channels::rpc::{
        BestTipWithProof, P2pChannelsRpcAction, P2pRpcRequest, P2pRpcResponse,
        StagedLedgerAuxAndPendingCoinbases,
    },
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    P2pNetworkYamuxAction, PeerId,
};

use super::*;

#[derive(Debug, thiserror::Error)]
enum RpcQueryError<'a> {
    #[error(transparent)]
    Read(#[from] RpcQueryReadError),
    #[error("unimplemeted rpc {}:{1}", String::from_utf8_lossy(.0))]
    Unimplemented(&'a [u8], Ver),
}

#[derive(Debug, thiserror::Error)]
enum RpcResponseError {
    #[error(transparent)]
    Read(#[from] RpcResponseReadError),
    #[error("rpc response {rpc_id} error: {error}")]
    Other { rpc_id: String, error: String },
}

fn rpc_query_effects<'a, Store, S>(
    peer_id: PeerId,
    QueryHeader { tag, version, id }: &'a QueryHeader,
    mut bytes: &[u8],
    store: &mut Store,
) -> Result<(), RpcQueryError<'a>>
where
    Store: crate::P2pStore<S>,
{
    let id = *id;
    match (tag.as_ref(), *version) {
        (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
            rpc::GetBestTipV2::query_payload(&mut bytes)?;
            store.dispatch(P2pChannelsRpcAction::RequestReceived {
                peer_id,
                id,
                request: Box::new(P2pRpcRequest::BestTipWithProof),
            });
        }
        (rpc::AnswerSyncLedgerQueryV2::NAME, rpc::AnswerSyncLedgerQueryV2::VERSION) => {
            let (hash, query) = rpc::AnswerSyncLedgerQueryV2::query_payload(&mut bytes)?;
            let hash = v2::LedgerHash::from(v2::MinaBaseLedgerHash0StableV1(hash));

            store.dispatch(P2pChannelsRpcAction::RequestReceived {
                peer_id,
                id,
                request: Box::new(P2pRpcRequest::LedgerQuery(hash, query)),
            });
        }
        (
            rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
            rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::VERSION,
        ) => {
            let hash =
                rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::query_payload(&mut bytes)?;
            let hash = v2::StateHash::from(v2::DataHashLibStateHashStableV1(hash));
            let request = Box::new(P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                hash,
            ));

            store.dispatch(P2pChannelsRpcAction::RequestReceived {
                peer_id,
                id,
                request,
            });
        }
        (rpc::GetTransitionChainV2::NAME, rpc::GetTransitionChainV2::VERSION) => {
            let hashes = rpc::GetTransitionChainV2::query_payload(&mut bytes)?;
            for hash in hashes {
                let hash = v2::StateHash::from(v2::DataHashLibStateHashStableV1(hash));

                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                    peer_id,
                    id,
                    request: Box::new(P2pRpcRequest::Block(hash)),
                });
            }
        }
        (rpc::GetSomeInitialPeersV1ForV2::NAME, rpc::GetSomeInitialPeersV1ForV2::VERSION) => {
            let () = rpc::GetSomeInitialPeersV1ForV2::query_payload(&mut bytes)?;
            store.dispatch(P2pChannelsRpcAction::RequestReceived {
                peer_id,
                id,
                request: Box::new(P2pRpcRequest::InitialPeers),
            });
        }
        (name, version) => return Err(RpcQueryError::Unimplemented(name, version)),
    }
    Ok(())
}

fn rpc_response_effects<Store, S>(
    peer_id: PeerId,
    QueryHeader { tag, version, id }: &QueryHeader,
    mut bytes: &[u8],
    store: &mut Store,
) -> Result<(), RpcResponseError>
where
    Store: crate::P2pStore<S>,
{
    let id = *id;
    match (tag.as_ref(), *version) {
        (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
            let response = rpc::GetBestTipV2::response_payload(&mut bytes)?
                .map(|resp| BestTipWithProof {
                    best_tip: resp.data.into(),
                    proof: (resp.proof.0, resp.proof.1.into()),
                })
                .map(P2pRpcResponse::BestTipWithProof)
                .map(Box::new);

            store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                peer_id,
                id,
                response,
            });
        }
        (rpc::AnswerSyncLedgerQueryV2::NAME, rpc::AnswerSyncLedgerQueryV2::VERSION) => {
            let response = Result::from(rpc::AnswerSyncLedgerQueryV2::response_payload(
                &mut bytes,
            )?)
            .map_err(|e| RpcResponseError::Other {
                rpc_id: rpc::AnswerSyncLedgerQueryV2::rpc_id(),
                error: e.to_string(),
            })?;
            let response = Some(Box::new(P2pRpcResponse::LedgerQuery(response)));

            store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                peer_id,
                id,
                response,
            });
        }
        (
            rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
            rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::VERSION,
        ) => {
            let response =
                rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::response_payload(&mut bytes)?;
            let response = response
                .map(|(scan_state, hash, pending_coinbase, needed_blocks)| {
                    let staged_ledger_hash = v2::MinaBaseLedgerHash0StableV1(hash).into();
                    Arc::new(StagedLedgerAuxAndPendingCoinbases {
                        scan_state,
                        staged_ledger_hash,
                        pending_coinbase,
                        needed_blocks,
                    })
                })
                .map(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock)
                .map(Box::new);

            store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                peer_id,
                id,
                response,
            });
        }
        (rpc::GetTransitionChainV2::NAME, rpc::GetTransitionChainV2::VERSION) => {
            let response = rpc::GetTransitionChainV2::response_payload(&mut bytes)?;
            match response {
                Some(response) if !response.is_empty() => {
                    for block in response {
                        let response = Some(Box::new(P2pRpcResponse::Block(Arc::new(block))));
                        store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                            peer_id,
                            id,
                            response,
                        });
                    }
                }
                _ => {
                    store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                        peer_id,
                        id,
                        response: None,
                    });
                }
            }
        }
        (rpc::GetSomeInitialPeersV1ForV2::NAME, rpc::GetSomeInitialPeersV1ForV2::VERSION) => {
            let response = rpc::GetSomeInitialPeersV1ForV2::response_payload(&mut bytes)?;
            if response.is_empty() {
                store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                    peer_id,
                    id,
                    response: None,
                });
            } else {
                let peers = response
                    .into_iter()
                    .filter_map(P2pConnectionOutgoingInitOpts::try_from_mina_rpc)
                    .collect();
                store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                    peer_id,
                    id,
                    response: Some(Box::new(P2pRpcResponse::InitialPeers(peers))),
                });
            }
        }
        _ => {}
    }
    Ok(())
}

impl P2pNetworkRpcAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let Some(state) = store.state().network.find_rpc_state(&self) else {
            error!(meta.time(); "cannot find stream for response: {self:?}");
            return;
        };

        let incoming = state.incoming.front().cloned();

        match self {
            P2pNetworkRpcAction::Init {
                addr,
                peer_id,
                stream_id,
                ..
            } => {
                store.dispatch(P2pNetworkRpcAction::OutgoingData {
                    addr,
                    peer_id,
                    stream_id,
                    data: RpcMessage::Handshake.into_bytes().into(),
                    fin: false,
                });
            }
            P2pNetworkRpcAction::IncomingData {
                addr,
                peer_id,
                stream_id,
                ..
            } => {
                if let Some(message) = incoming {
                    store.dispatch(P2pNetworkRpcAction::IncomingMessage {
                        addr,
                        peer_id,
                        stream_id,
                        message,
                    });
                }
            }
            P2pNetworkRpcAction::IncomingMessage {
                addr,
                peer_id,
                stream_id,
                message,
            } => {
                match &message {
                    RpcMessage::Handshake => {
                        if !state.is_incoming {
                            store.dispatch(P2pChannelsRpcAction::Ready { peer_id });
                        }
                    }
                    RpcMessage::Heartbeat => {}
                    RpcMessage::Query { header, bytes } => {
                        if let Err(e) = rpc_query_effects(peer_id, header, bytes, store) {
                            store.dispatch(P2pDisconnectionAction::Init {
                                peer_id,
                                reason: P2pDisconnectionReason::P2pChannelReceiveFailed(
                                    e.to_string(),
                                ),
                            });
                        }
                    }
                    RpcMessage::Response {
                        header: ResponseHeader { id },
                        bytes,
                    } => {
                        let query_header = match state.pending.as_ref() {
                            Some(header) if &header.id == id => header.clone(),
                            Some(header) => {
                                error!(meta.time(); "received response with it {} while expecting {id}", header.id);
                                return;
                            }
                            None => {
                                error!(meta.time(); "received response while no query is sent");
                                return;
                            }
                        };
                        // unset pending
                        store.dispatch(P2pNetworkRpcAction::PrunePending { peer_id, stream_id });

                        if let Err(e) = rpc_response_effects(peer_id, &query_header, bytes, store) {
                            store.dispatch(P2pDisconnectionAction::Init {
                                peer_id,
                                reason: P2pDisconnectionReason::P2pChannelReceiveFailed(
                                    e.to_string(),
                                ),
                            });
                        }
                    }
                }

                if let Some(message) = incoming {
                    store.dispatch(P2pNetworkRpcAction::IncomingMessage {
                        addr,
                        peer_id,
                        stream_id,
                        message,
                    });
                }
            }
            P2pNetworkRpcAction::PrunePending { .. } => {}
            P2pNetworkRpcAction::HeartbeatSend {
                addr,
                peer_id,
                stream_id,
            } => {
                store.dispatch(P2pNetworkRpcAction::OutgoingData {
                    addr,
                    peer_id,
                    stream_id,
                    data: RpcMessage::Heartbeat.into_bytes().into(),
                    fin: false,
                });
            }
            P2pNetworkRpcAction::OutgoingQuery {
                peer_id,
                query,
                data,
            } => {
                store.dispatch(P2pNetworkRpcAction::OutgoingData {
                    addr: state.addr,
                    peer_id,
                    stream_id: state.stream_id,
                    data: RpcMessage::Query {
                        header: query.clone(),
                        bytes: data.clone(),
                    }
                    .into_bytes()
                    .into(),
                    fin: false,
                });
            }
            P2pNetworkRpcAction::OutgoingResponse {
                peer_id,
                response,
                data,
            } => {
                if !matches!(state.pending, Some(QueryHeader { id, .. }) if id == response.id) {
                    openmina_core::error!(meta.time(); "pending query does not match the response");
                    return;
                }
                let stream_id = state.stream_id;
                let addr = state.addr;
                store.dispatch(P2pNetworkRpcAction::PrunePending { peer_id, stream_id });
                store.dispatch(P2pNetworkRpcAction::OutgoingData {
                    addr,
                    peer_id,
                    stream_id,
                    data: RpcMessage::Response {
                        header: response,
                        bytes: data,
                    }
                    .into_bytes()
                    .into(),
                    fin: false,
                });
            }
            P2pNetworkRpcAction::OutgoingData {
                addr,
                stream_id,
                mut data,
                ..
            } => {
                fuzz_maybe!(&mut data, crate::fuzzer::mutate_rpc_data);
                let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data,
                    flags,
                });
            }
        }
    }
}

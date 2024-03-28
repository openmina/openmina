use std::sync::Arc;

use binprot::BinProtRead;
use mina_p2p_messages::{
    rpc,
    rpc_kernel::{PayloadBinprotReader, QueryPayload, RpcMethod},
    v2,
    versioned::Ver,
};
use openmina_core::error;

use crate::{
    channels::rpc::{
        BestTipWithProof, P2pChannelsRpcAction, P2pRpcId, P2pRpcRequest, P2pRpcResponse,
        StagedLedgerAuxAndPendingCoinbases,
    },
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    P2pNetworkYamuxAction, PeerId,
};

use super::*;

fn rpc_response_parser<M>(mut bytes: &[u8]) -> Result<M::Response, String>
where
    M: PayloadBinprotReader,
{
    Ok(M::response_payload(&mut bytes)
        .map_err(|e| format!("failed to parse rpc {}:{}: {e}", M::NAME_STR, M::VERSION))?
        .map_err(|e| {
            format!(
                "received failure for rpc {}:{}: {e:?}",
                M::NAME_STR,
                M::VERSION
            )
        })?)
}

fn rpc_response_effects<Store, S>(
    tag: &[u8],
    version: Ver,
    id: P2pRpcId,
    bytes: &[u8],
    peer_id: PeerId,
    store: &mut Store,
) -> Result<(), String>
where
    Store: crate::P2pStore<S>,
{
    match (tag, version) {
        (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
            let response = rpc_response_parser::<rpc::GetBestTipV2>(&bytes)?
                .map(|resp| BestTipWithProof {
                    best_tip: resp.data.into(),
                    proof: (resp.proof.0, resp.proof.1.into()),
                })
                .map(P2pRpcResponse::BestTipWithProof);

            store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                peer_id,
                id,
                response,
            });
        }
        (rpc::AnswerSyncLedgerQueryV2::NAME, rpc::AnswerSyncLedgerQueryV2::VERSION) => {
            let response =
                Result::from(rpc_response_parser::<rpc::AnswerSyncLedgerQueryV2>(&bytes)?)
                    .map_err(|e| {
                        format!(
                            "rpc method {}:{} returned error {e}",
                            rpc::AnswerSyncLedgerQueryV2::NAME_STR,
                            rpc::AnswerSyncLedgerQueryV2::VERSION
                        )
                    })?;
            let response = Some(P2pRpcResponse::LedgerQuery(response));

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
                rpc_response_parser::<rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>(&bytes)?;
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
                .map(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock);

            store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                peer_id,
                id,
                response,
            });
        }
        (rpc::GetTransitionChainV2::NAME, rpc::GetTransitionChainV2::VERSION) => {
            let response = rpc_response_parser::<rpc::GetTransitionChainV2>(&bytes)?;
            match response {
                Some(response) if !response.is_empty() => {
                    for block in response {
                        let response = Some(P2pRpcResponse::Block(Arc::new(block)));
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
            let response = rpc_response_parser::<rpc::GetSomeInitialPeersV1ForV2>(&bytes)?;
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
                    response: Some(P2pRpcResponse::InitialPeers(peers)),
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
            return;
        };

        let incoming = state.incoming.front().cloned();

        match self {
            Self::Init {
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
            Self::IncomingData {
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
            Self::IncomingMessage {
                addr,
                peer_id,
                stream_id,
                message,
            } => {
                // TODO: process
                match &message {
                    RpcMessage::Handshake => {
                        if !state.is_incoming {
                            store.dispatch(P2pChannelsRpcAction::Ready { peer_id });
                        }
                    }
                    RpcMessage::Heartbeat => {
                        store.dispatch(P2pNetworkRpcAction::OutgoingData {
                            addr,
                            peer_id,
                            stream_id,
                            data: RpcMessage::Heartbeat.into_bytes().into(),
                            fin: false,
                        });
                    }
                    RpcMessage::Query { header, bytes } => {
                        fn parse_q<M: RpcMethod>(bytes: &[u8]) -> Result<M::Query, String> {
                            let mut bytes = bytes;
                            <QueryPayload<M::Query> as BinProtRead>::binprot_read(&mut bytes)
                                .map(|x| x.0)
                                .map_err(|err| format!("response {} {}", M::NAME_STR, err))
                        }

                        match (header.tag.as_ref(), header.version) {
                            (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
                                let Ok(()) = parse_q::<rpc::GetBestTipV2>(&bytes) else {
                                    // TODO: close the stream
                                    panic!();
                                };
                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id,
                                    id: header.id,
                                    request: P2pRpcRequest::BestTipWithProof,
                                });
                            }
                            (
                                rpc::AnswerSyncLedgerQueryV2::NAME,
                                rpc::AnswerSyncLedgerQueryV2::VERSION,
                            ) => {
                                let Ok((hash, query)) =
                                    parse_q::<rpc::AnswerSyncLedgerQueryV2>(&bytes)
                                else {
                                    // TODO: close the stream
                                    panic!();
                                };

                                let hash =
                                    v2::LedgerHash::from(v2::MinaBaseLedgerHash0StableV1(hash));

                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id,
                                    id: header.id,
                                    request: P2pRpcRequest::LedgerQuery(hash, query),
                                });
                            }
                            (
                                rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
                                rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::VERSION,
                            ) => {
                                let Ok(hash) = parse_q::<
                                    rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2,
                                >(&bytes) else {
                                    // TODO: close the stream
                                    panic!();
                                };

                                let hash =
                                    v2::StateHash::from(v2::DataHashLibStateHashStableV1(hash));
                                let request =
                                    P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(hash);

                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id,
                                    id: header.id,
                                    request,
                                });
                            }
                            (
                                rpc::GetTransitionChainV2::NAME,
                                rpc::GetTransitionChainV2::VERSION,
                            ) => {
                                let Ok(hashes) = parse_q::<rpc::GetTransitionChainV2>(&bytes)
                                else {
                                    // TODO: close the stream
                                    panic!();
                                };

                                for hash in hashes {
                                    let hash =
                                        v2::StateHash::from(v2::DataHashLibStateHashStableV1(hash));

                                    store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                        peer_id,
                                        id: header.id,
                                        request: P2pRpcRequest::Block(hash),
                                    });
                                }
                            }
                            (
                                rpc::GetSomeInitialPeersV1ForV2::NAME,
                                rpc::GetSomeInitialPeersV1ForV2::VERSION,
                            ) => {
                                let Ok(()) = parse_q::<rpc::GetSomeInitialPeersV1ForV2>(&bytes)
                                else {
                                    // TODO: close the stream
                                    panic!();
                                };

                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id,
                                    id: header.id,
                                    request: P2pRpcRequest::InitialPeers,
                                });
                            }
                            _ => {}
                        }
                    }
                    RpcMessage::Response { header, bytes } => {
                        let (id, tag, version) = {
                            let Some((id, (tag, version))) = state.pending.clone() else {
                                error!(meta.time(); "no query");
                                return;
                            };
                            if id != header.id {
                                error!(meta.time(); "invalid response id");
                                return;
                            };
                            (id, tag, version)
                        };
                        // unset pending
                        store.dispatch(P2pNetworkRpcAction::PrunePending { peer_id, stream_id });

                        if let Err(e) =
                            rpc_response_effects(tag.as_ref(), version, id, &bytes, peer_id, store)
                        {
                            store.dispatch(P2pDisconnectionAction::Init {
                                peer_id,
                                reason: P2pDisconnectionReason::P2pChannelReceiveFailed(e),
                            });
                        }
                        // TODO: dispatch further action
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
            Self::PrunePending { .. } => {}
            Self::OutgoingQuery {
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
            Self::OutgoingResponse {
                peer_id,
                response,
                data,
            } => {
                store.dispatch(P2pNetworkRpcAction::OutgoingData {
                    addr: state.addr,
                    peer_id,
                    stream_id: state.stream_id,
                    data: RpcMessage::Response {
                        header: response,
                        bytes: data,
                    }
                    .into_bytes()
                    .into(),
                    fin: false,
                });
            }
            Self::OutgoingData {
                addr,
                stream_id,
                data,
                ..
            } => {
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr,
                    stream_id,
                    data,
                    fin: false,
                });
            }
        }
    }
}

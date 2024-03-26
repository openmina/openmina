use std::sync::Arc;

use binprot::BinProtRead;
use mina_p2p_messages::{
    rpc,
    rpc_kernel::{Error as RpcError, NeedsLength, QueryPayload, ResponsePayload, RpcMethod},
    v2,
};
use openmina_core::warn;

use crate::{
    channels::rpc::{
        BestTipWithProof, P2pChannelsRpcAction, P2pRpcRequest, P2pRpcResponse,
        StagedLedgerAuxAndPendingCoinbases,
    },
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    P2pNetworkYamuxAction,
};

use super::*;

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
                                .map_err(|err| format!("response {} {}", M::NAME, err))
                        }

                        match (header.tag.to_string_lossy().as_str(), header.version) {
                            (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
                                let Ok(()) = parse_q::<rpc::GetBestTipV2>(&bytes) else {
                                    // TODO: close the stream
                                    panic!();
                                };
                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id,
                                    id: header.id as u32,
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
                                    id: header.id as u32,
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
                                    id: header.id as u32,
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
                                        id: header.id as u32,
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
                                    id: header.id as u32,
                                    request: P2pRpcRequest::InitialPeers,
                                });
                            }
                            _ => {}
                        }
                    }
                    RpcMessage::Response { header, bytes } => {
                        fn parse_r<M: RpcMethod>(
                            mut bytes: &[u8],
                            time: redux::Timestamp,
                        ) -> Option<Result<M::Response, RpcError>> {
                            match <ResponsePayload<M::Response> as BinProtRead>::binprot_read(&mut bytes)
                                .map(|x| x.0.map(|NeedsLength(x)| x))
                                 {
                                    Ok(v) => Some(v),
                                    Err(e) => {
                                        warn!(time; "response {} {}", M::NAME, e);
                                        None
                                    },
                                 }
                        }

                        if let Some((_, (tag, version))) = &state.pending {
                            if let Ok(tag) = std::str::from_utf8(tag.as_ref()) {
                                match (tag, *version) {
                                    (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
                                        let Some(response) = parse_r::<rpc::GetBestTipV2>(&bytes, meta.time()) else {
                                            return
                                        };
                                        let response = response
                                            .ok()
                                            .flatten()
                                            .map(|resp| BestTipWithProof {
                                                best_tip: resp.data.into(),
                                                proof: (resp.proof.0, resp.proof.1.into()),
                                            })
                                            .map(P2pRpcResponse::BestTipWithProof);

                                        store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                                            peer_id,
                                            id: header.id as _,
                                            response,
                                        });
                                    }
                                    (
                                        rpc::AnswerSyncLedgerQueryV2::NAME,
                                        rpc::AnswerSyncLedgerQueryV2::VERSION,
                                    ) => {
                                        let Some(response) = parse_r::<rpc::AnswerSyncLedgerQueryV2>(&bytes, meta.time()) else {
                                            return;
                                        };

                                        let response = response
                                            .ok()
                                            .map(|x| x.0.ok())
                                            .flatten()
                                            .map(P2pRpcResponse::LedgerQuery);

                                        store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                                            peer_id,
                                            id: header.id as _,
                                            response,
                                        });
                                    }
                                    (
                                        rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
                                        rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::VERSION,
                                    ) => {
                                        let Some(response) = parse_r::<rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2>(&bytes, meta.time()) else {
                                            return;
                                        };
                                        let response = response
                                        .ok()
                                        .flatten()
                                        .map(|(scan_state, hash, pending_coinbase, needed_blocks)| {
                                            let staged_ledger_hash =
                                                v2::MinaBaseLedgerHash0StableV1(hash).into();
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
                                            id: header.id as _,
                                            response,
                                        });
                                    }
                                    (
                                        rpc::GetTransitionChainV2::NAME,
                                        rpc::GetTransitionChainV2::VERSION,
                                    ) => {
                                        type Method = rpc::GetTransitionChainV2;
                                        let Some(response) = parse_r::<Method>(&bytes, meta.time()) else {
                                            return;
                                        };
                                        let response = response.ok().flatten().unwrap_or_default();

                                        if response.is_empty() {
                                            store.dispatch(
                                                P2pChannelsRpcAction::ResponseReceived {
                                                    peer_id,
                                                    id: header.id as _,
                                                    response: None,
                                                },
                                            );
                                        } else {
                                            for block in response {
                                                let response =
                                                    Some(P2pRpcResponse::Block(Arc::new(block)));
                                                store.dispatch(
                                                    P2pChannelsRpcAction::ResponseReceived {
                                                        peer_id,
                                                        id: header.id as _,
                                                        response,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                    (
                                        rpc::GetSomeInitialPeersV1ForV2::NAME,
                                        rpc::GetSomeInitialPeersV1ForV2::VERSION,
                                    ) => {
                                        type Method = rpc::GetSomeInitialPeersV1ForV2;
                                        let Some(response) = parse_r::<Method>(&bytes, meta.time()) else {
                                            // TODO: close the stream
                                            panic!();
                                        };
                                        let Ok(response) = response else { return };
                                        if response.is_empty() {
                                            store.dispatch(
                                                P2pChannelsRpcAction::ResponseReceived {
                                                    peer_id,
                                                    id: header.id as _,
                                                    response: None,
                                                },
                                            );
                                        } else {
                                            let peers = response.into_iter().filter_map(P2pConnectionOutgoingInitOpts::try_from_mina_rpc).collect();
                                            store.dispatch(
                                                P2pChannelsRpcAction::ResponseReceived {
                                                    peer_id,
                                                    id: header.id as _,
                                                    response: Some(P2pRpcResponse::InitialPeers(
                                                        peers,
                                                    )),
                                                },
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
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

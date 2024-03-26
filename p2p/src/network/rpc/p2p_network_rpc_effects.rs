use std::sync::Arc;

use binprot::BinProtRead;
use mina_p2p_messages::{
    rpc,
    rpc_kernel::{Error as RpcError, NeedsLength, QueryPayload, ResponsePayload, RpcMethod},
    v2,
};

use crate::{
    channels::rpc::{
        BestTipWithProof, P2pChannelsRpcAction, P2pRpcRequest, P2pRpcResponse,
        StagedLedgerAuxAndPendingCoinbases,
    },
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    P2pNetworkYamuxOutgoingDataAction,
};

use super::*;

impl P2pNetworkRpcAction {
    pub fn effects<Store, S>(&self, _: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let Some(state) = store.state().network.find_rpc_state(self) else {
            return;
        };

        let incoming = state.incoming.front().cloned();

        match self {
            Self::Init(a) => {
                store.dispatch(P2pNetworkRpcOutgoingDataAction {
                    addr: a.addr,
                    peer_id: a.peer_id,
                    stream_id: a.stream_id,
                    data: RpcMessage::Handshake.into_bytes().into(),
                    fin: false,
                });
            }
            Self::IncomingData(a) => {
                if let Some(message) = incoming {
                    store.dispatch(P2pNetworkRpcIncomingMessageAction {
                        addr: a.addr,
                        peer_id: a.peer_id,
                        stream_id: a.stream_id,
                        message,
                    });

                    store.dispatch(P2pChannelsRpcAction::Ready { peer_id: a.peer_id });
                }
            }
            Self::IncomingMessage(a) => {
                // TODO: process
                match &a.message {
                    RpcMessage::Handshake => {
                        if !state.is_incoming {
                            // let mut v = vec![];

                            // type Payload =
                            //     QueryPayload<<rpc::VersionedRpcMenuV1 as RpcMethod>::Query>;
                            // <Payload as BinProtWrite>::binprot_write(&NeedsLength(()), &mut v)
                            //     .unwrap_or_default();

                            // store.dispatch(P2pNetworkRpcOutgoingQueryAction {
                            //     peer_id: a.peer_id,
                            //     query: QueryHeader {
                            //         tag: rpc::VersionedRpcMenuV1::NAME.into(),
                            //         version: rpc::VersionedRpcMenuV1::VERSION,
                            //         id: state.last_id,
                            //     },
                            //     data: v.into(),
                            // });
                        }
                    }
                    RpcMessage::Heartbeat => {
                        store.dispatch(P2pNetworkRpcOutgoingDataAction {
                            addr: a.addr,
                            peer_id: a.peer_id,
                            stream_id: a.stream_id,
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
                                let Ok(()) = parse_q::<rpc::GetBestTipV2>(bytes) else {
                                    // TODO: close the stream
                                    panic!();
                                };
                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id: a.peer_id,
                                    id: header.id as u32,
                                    request: P2pRpcRequest::BestTipWithProof,
                                });
                            }
                            (
                                rpc::AnswerSyncLedgerQueryV2::NAME,
                                rpc::AnswerSyncLedgerQueryV2::VERSION,
                            ) => {
                                let Ok((hash, query)) =
                                    parse_q::<rpc::AnswerSyncLedgerQueryV2>(bytes)
                                else {
                                    // TODO: close the stream
                                    panic!();
                                };

                                let hash =
                                    v2::LedgerHash::from(v2::MinaBaseLedgerHash0StableV1(hash));

                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id: a.peer_id,
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
                                >(bytes) else {
                                    // TODO: close the stream
                                    panic!();
                                };

                                let hash =
                                    v2::StateHash::from(v2::DataHashLibStateHashStableV1(hash));
                                let request =
                                    P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(hash);

                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id: a.peer_id,
                                    id: header.id as u32,
                                    request,
                                });
                            }
                            (
                                rpc::GetTransitionChainV2::NAME,
                                rpc::GetTransitionChainV2::VERSION,
                            ) => {
                                let Ok(hashes) = parse_q::<rpc::GetTransitionChainV2>(bytes) else {
                                    // TODO: close the stream
                                    panic!();
                                };

                                for hash in hashes {
                                    let hash =
                                        v2::StateHash::from(v2::DataHashLibStateHashStableV1(hash));

                                    store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                        peer_id: a.peer_id,
                                        id: header.id as u32,
                                        request: P2pRpcRequest::Block(hash),
                                    });
                                }
                            }
                            (
                                rpc::GetSomeInitialPeersV1ForV2::NAME,
                                rpc::GetSomeInitialPeersV1ForV2::VERSION,
                            ) => {
                                let Ok(()) = parse_q::<rpc::GetSomeInitialPeersV1ForV2>(bytes)
                                else {
                                    // TODO: close the stream
                                    panic!();
                                };

                                store.dispatch(P2pChannelsRpcAction::RequestReceived {
                                    peer_id: a.peer_id,
                                    id: header.id as u32,
                                    request: P2pRpcRequest::InitialPeers,
                                });
                            }
                            _ => {}
                        }
                    }
                    RpcMessage::Response { header, bytes } => {
                        fn parse_r<M: RpcMethod>(
                            bytes: &[u8],
                        ) -> Result<Result<M::Response, RpcError>, String> {
                            let mut bytes = bytes;
                            <ResponsePayload<M::Response> as BinProtRead>::binprot_read(&mut bytes)
                                .map(|x| x.0.map(|NeedsLength(x)| x))
                                .map_err(|err| format!("response {} {}", M::NAME, err))
                        }

                        if let Some((_, (tag, version))) = &state.pending {
                            if let Ok(tag) = std::str::from_utf8(tag.as_ref()) {
                                match (tag, *version) {
                                    (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
                                        let Ok(response) = parse_r::<rpc::GetBestTipV2>(bytes)
                                        else {
                                            // TODO: close the stream
                                            panic!();
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
                                            peer_id: a.peer_id,
                                            id: header.id as _,
                                            response,
                                        });
                                    }
                                    (
                                        rpc::AnswerSyncLedgerQueryV2::NAME,
                                        rpc::AnswerSyncLedgerQueryV2::VERSION,
                                    ) => {
                                        let Ok(response) =
                                            parse_r::<rpc::AnswerSyncLedgerQueryV2>(bytes)
                                        else {
                                            // TODO: close the stream
                                            panic!();
                                        };

                                        let response = response
                                            .ok()
                                            .and_then(|x| x.0.ok())
                                            .map(P2pRpcResponse::LedgerQuery);

                                        store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                                            peer_id: a.peer_id,
                                            id: header.id as _,
                                            response,
                                        });
                                    }
                                    (
                                        rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
                                        rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::VERSION,
                                    ) => {
                                        type Method =
                                            rpc::GetStagedLedgerAuxAndPendingCoinbasesAtHashV2;
                                        let Ok(response) = parse_r::<Method>(bytes) else {
                                            // TODO: close the stream
                                            panic!();
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
                                            peer_id: a.peer_id,
                                            id: header.id as _,
                                            response,
                                        });
                                    }
                                    (
                                        rpc::GetTransitionChainV2::NAME,
                                        rpc::GetTransitionChainV2::VERSION,
                                    ) => {
                                        type Method = rpc::GetTransitionChainV2;
                                        let Ok(response) = parse_r::<Method>(bytes) else {
                                            // TODO: close the stream
                                            panic!();
                                        };
                                        let response = response.ok().flatten().unwrap_or_default();

                                        if response.is_empty() {
                                            store.dispatch(
                                                P2pChannelsRpcAction::ResponseReceived {
                                                    peer_id: a.peer_id,
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
                                                        peer_id: a.peer_id,
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
                                        let Ok(response) = parse_r::<Method>(bytes) else {
                                            // TODO: close the stream
                                            panic!();
                                        };
                                        let Ok(response) = response else { return };
                                        if response.is_empty() {
                                            store.dispatch(
                                                P2pChannelsRpcAction::ResponseReceived {
                                                    peer_id: a.peer_id,
                                                    id: header.id as _,
                                                    response: None,
                                                },
                                            );
                                        } else {
                                            let peers = response.into_iter().filter_map(P2pConnectionOutgoingInitOpts::try_from_mina_rpc).collect();
                                            store.dispatch(
                                                P2pChannelsRpcAction::ResponseReceived {
                                                    peer_id: a.peer_id,
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
                    store.dispatch(P2pNetworkRpcIncomingMessageAction {
                        addr: a.addr,
                        peer_id: a.peer_id,
                        stream_id: a.stream_id,
                        message,
                    });
                }
            }
            Self::OutgoingQuery(a) => {
                store.dispatch(P2pNetworkRpcOutgoingDataAction {
                    addr: state.addr,
                    peer_id: a.peer_id,
                    stream_id: state.stream_id,
                    data: RpcMessage::Query {
                        header: a.query.clone(),
                        bytes: a.data.clone(),
                    }
                    .into_bytes()
                    .into(),
                    fin: false,
                });
            }
            Self::OutgoingResponse(a) => {
                store.dispatch(P2pNetworkRpcOutgoingDataAction {
                    addr: state.addr,
                    peer_id: a.peer_id,
                    stream_id: state.stream_id,
                    data: RpcMessage::Response {
                        header: a.response.clone(),
                        bytes: a.data.clone(),
                    }
                    .into_bytes()
                    .into(),
                    fin: false,
                });
            }
            Self::OutgoingData(a) => {
                store.dispatch(P2pNetworkYamuxOutgoingDataAction {
                    addr: a.addr,
                    stream_id: a.stream_id,
                    data: a.data.clone(),
                    fin: false,
                });
            }
        }
    }
}

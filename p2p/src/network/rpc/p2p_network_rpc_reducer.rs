use std::sync::Arc;

use binprot::BinProtRead;
use mina_p2p_messages::{
    rpc,
    rpc_kernel::{
        MessageHeader, PayloadBinprotReader as _, QueryHeader, ResponseHeader, RpcMethod,
        RpcQueryReadError, RpcResponseReadError,
    },
    v2,
    versioned::Ver,
};
use openmina_core::{bug_condition, error, fuzz_maybe, fuzzed_maybe, Substate};
use redux::Dispatcher;

use crate::{
    channels::rpc::{
        BestTipWithProof, P2pChannelsRpcAction, P2pRpcRequest, P2pRpcResponse,
        StagedLedgerAuxAndPendingCoinbases,
    },
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    Data, Limit, P2pLimits, P2pNetworkState, P2pNetworkYamuxAction, PeerId,
};

use self::p2p_network_rpc_state::P2pNetworkRpcError;

use super::*;

impl P2pNetworkRpcState {
    /// Substate is accessed
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, P2pNetworkState>,
        action: redux::ActionWithMeta<&P2pNetworkRpcAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let rpc_state = state_context
            .get_substate_mut()?
            .find_rpc_state_mut(action.action())
            .ok_or_else(|| format!("RPC state not found for action: {:?}", action.action()))
            .inspect_err(|e| bug_condition!("{}", e))?;

        let (action, meta) = action.split();
        match action {
            P2pNetworkRpcAction::Init {
                incoming,
                addr,
                peer_id,
                stream_id,
            } => {
                rpc_state.is_incoming = *incoming;

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkRpcAction::OutgoingData {
                    addr: *addr,
                    peer_id: *peer_id,
                    stream_id: *stream_id,
                    data: Data::from(RpcMessage::Handshake.into_bytes()),
                    fin: false,
                });
                Ok(())
            }
            P2pNetworkRpcAction::IncomingData {
                data,
                addr,
                peer_id,
                stream_id,
            } => {
                rpc_state.buffer.extend_from_slice(data);
                let mut offset = 0;
                // TODO(akoptelov): there shouldn't be the case where we have multiple incoming messages at once (or at least other than heartbeat)
                loop {
                    let Some(buf) = &rpc_state.buffer.get(offset..) else {
                        bug_condition!("Invalid range `buffer[{offset}..]`");
                        return Ok(());
                    };
                    if let Some(len_bytes) = buf.get(..8).and_then(|s| s.try_into().ok()) {
                        let len = u64::from_le_bytes(len_bytes) as usize;
                        if let Err(err) = rpc_state.check_rpc_limit(len, limits) {
                            rpc_state.error = Some(err);
                            break;
                        }
                        if let Some(mut slice) = buf.get(8..(8 + len)) {
                            offset += 8 + len;
                            let msg = match MessageHeader::binprot_read(&mut slice) {
                                Ok(MessageHeader::Heartbeat) => RpcMessage::Heartbeat,
                                Ok(MessageHeader::Response(h))
                                    if h.id == u64::from_le_bytes(*b"RPC\x00\x00\x00\x00\x00") =>
                                {
                                    RpcMessage::Handshake
                                }
                                Ok(MessageHeader::Query(header)) => RpcMessage::Query {
                                    header,
                                    bytes: slice.to_vec().into(),
                                },
                                Ok(MessageHeader::Response(header)) => RpcMessage::Response {
                                    header,
                                    bytes: slice.to_vec().into(),
                                },
                                Err(err) => {
                                    rpc_state.error =
                                        Some(P2pNetworkRpcError::Binprot(err.to_string()));
                                    continue;
                                }
                            };
                            rpc_state.incoming.push_back(msg);
                            continue;
                        }
                    }

                    if offset != 0 {
                        let Some(buf) = rpc_state.buffer.get(offset..) else {
                            bug_condition!("Invalid range `buffer[{offset}..]`");
                            return Ok(());
                        };
                        rpc_state.buffer = buf.to_vec();
                    }
                    break;
                }

                let incoming = rpc_state.incoming.front().cloned();
                let dispatcher = state_context.into_dispatcher();

                if let Some(message) = incoming {
                    dispatcher.push(P2pNetworkRpcAction::IncomingMessage {
                        addr: *addr,
                        peer_id: *peer_id,
                        stream_id: *stream_id,
                        message,
                    })
                }

                Ok(())
            }
            action @ P2pNetworkRpcAction::IncomingMessage {
                message,
                addr,
                peer_id,
                stream_id,
            } => {
                if let RpcMessage::Response { header, .. } = message {
                    if let Some(QueryHeader { id, tag, version }) = &rpc_state.pending {
                        *rpc_state
                            .total_stats
                            .entry((tag.clone(), *version))
                            .or_default() += 1;
                        if id != &header.id {
                            error!(meta.time(); "receiving response with wrong id: {}", header.id);
                        }
                    } else {
                        error!(meta.time(); "receiving response without query");
                    }
                } else if let RpcMessage::Query { header, .. } = message {
                    if rpc_state.pending.is_none() {
                        rpc_state.pending = Some(header.clone());
                    } else {
                        error!(meta.time(); "receiving query while another query is pending");
                    }
                }

                rpc_state.incoming.pop_front();

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let network_state: &P2pNetworkState = state.substate()?;
                let state = network_state
                    .find_rpc_state(action)
                    .ok_or_else(|| format!("RPC state not found for action: {:?}", action))?;

                match &message {
                    RpcMessage::Handshake => {
                        if !state.is_incoming {
                            dispatcher.push(P2pChannelsRpcAction::Ready { peer_id: *peer_id });
                        }
                    }
                    RpcMessage::Heartbeat => {}
                    RpcMessage::Query { header, bytes } => {
                        if let Err(e) = dispatch_rpc_query(*peer_id, header, bytes, dispatcher) {
                            dispatcher.push(P2pDisconnectionAction::Init {
                                peer_id: *peer_id,
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
                                return Ok(());
                            }
                            None => {
                                error!(meta.time(); "received response while no query is sent");
                                return Ok(());
                            }
                        };
                        // unset pending
                        dispatcher.push(P2pNetworkRpcAction::PrunePending {
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                        });

                        if let Err(e) =
                            dispatch_rpc_response(*peer_id, &query_header, bytes, dispatcher)
                        {
                            dispatcher.push(P2pDisconnectionAction::Init {
                                peer_id: *peer_id,
                                reason: P2pDisconnectionReason::P2pChannelReceiveFailed(
                                    e.to_string(),
                                ),
                            });
                        }
                    }
                }

                if let Some(message) = state.incoming.front().cloned() {
                    dispatcher.push(P2pNetworkRpcAction::IncomingMessage {
                        addr: *addr,
                        peer_id: *peer_id,
                        stream_id: *stream_id,
                        message,
                    });
                }
                Ok(())
            }
            P2pNetworkRpcAction::PrunePending { .. } => {
                rpc_state.pending = None;
                Ok(())
            }
            P2pNetworkRpcAction::HeartbeatSend {
                addr,
                peer_id,
                stream_id,
            } => {
                rpc_state.last_heartbeat_sent = Some(meta.time());

                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pNetworkRpcAction::OutgoingData {
                    addr: *addr,
                    peer_id: *peer_id,
                    stream_id: *stream_id,
                    data: Data::from(RpcMessage::Heartbeat.into_bytes()),
                    fin: false,
                });

                Ok(())
            }
            P2pNetworkRpcAction::OutgoingQuery {
                query,
                data,
                peer_id,
            } => {
                rpc_state.last_id = query.id;
                rpc_state.pending = Some(query.clone());

                let addr = rpc_state.addr;
                let stream_id = rpc_state.stream_id;
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkRpcAction::OutgoingData {
                    addr,
                    peer_id: *peer_id,
                    stream_id,
                    data: Data::from(
                        RpcMessage::Query {
                            header: query.clone(),
                            bytes: data.clone(),
                        }
                        .into_bytes(),
                    ),
                    fin: false,
                });

                Ok(())
            }
            P2pNetworkRpcAction::OutgoingData {
                addr,
                stream_id,
                data,
                ..
            } => {
                let dispatcher = state_context.into_dispatcher();
                let mut data = data.clone();
                fuzz_maybe!(&mut data, crate::fuzzer::mutate_rpc_data);
                let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

                dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                    addr: *addr,
                    stream_id: *stream_id,
                    data,
                    flags,
                });

                Ok(())
            }
            P2pNetworkRpcAction::OutgoingResponse {
                peer_id,
                response,
                data,
            } => {
                if !matches!(rpc_state.pending, Some(QueryHeader { id, .. }) if id == response.id) {
                    bug_condition!("pending query does not match the response");
                    return Ok(());
                }
                let stream_id = rpc_state.stream_id;
                let addr = rpc_state.addr;
                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pNetworkRpcAction::PrunePending {
                    peer_id: *peer_id,
                    stream_id,
                });
                dispatcher.push(P2pNetworkRpcAction::OutgoingData {
                    addr,
                    peer_id: *peer_id,
                    stream_id,
                    data: Data::from(
                        RpcMessage::Response {
                            header: response.clone(),
                            bytes: data.clone(),
                        }
                        .into_bytes(),
                    ),
                    fin: false,
                });
                Ok(())
            }
        }
    }

    fn check_rpc_limit(&self, len: usize, limits: &P2pLimits) -> Result<(), P2pNetworkRpcError> {
        let (limit, kind): (_, &[u8]) = if self.is_incoming {
            // only requests are allowed
            (limits.rpc_query(), b"<query>")
        } else if let Some(QueryHeader { tag, .. }) = self.pending.as_ref() {
            use mina_p2p_messages::rpc::*;
            match tag.as_ref() {
                GetBestTipV2::NAME => (limits.rpc_get_best_tip(), GetBestTipV2::NAME),
                AnswerSyncLedgerQueryV2::NAME => (
                    limits.rpc_answer_sync_ledger_query(),
                    AnswerSyncLedgerQueryV2::NAME,
                ),
                GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME => (
                    limits.rpc_get_staged_ledger(),
                    GetStagedLedgerAuxAndPendingCoinbasesAtHashV2::NAME,
                ),
                GetTransitionChainV2::NAME => (
                    limits.rpc_get_transition_chain(),
                    GetTransitionChainV2::NAME,
                ),
                GetSomeInitialPeersV1ForV2::NAME => (
                    limits.rpc_get_some_initial_peers(),
                    GetSomeInitialPeersV1ForV2::NAME,
                ),
                _ => (Limit::Some(0), b"<unimplemented>"),
            }
        } else {
            (limits.rpc_service_message(), b"<service_messages>")
        };
        let kind = String::from_utf8_lossy(kind);
        if len > limit {
            Err(P2pNetworkRpcError::Limit(kind.into_owned(), len, limit))
        } else {
            Ok(())
        }
    }
}

fn dispatch_rpc_query<'a, State, Action>(
    peer_id: PeerId,
    QueryHeader { tag, version, id }: &'a QueryHeader,
    mut bytes: &[u8],
    dispatcher: &mut Dispatcher<Action, State>,
) -> Result<(), RpcQueryError<'a>>
where
    State: crate::P2pStateTrait,
    Action: crate::P2pActionTrait<State>,
{
    let id = *id;
    match (tag.as_ref(), *version) {
        (rpc::GetBestTipV2::NAME, rpc::GetBestTipV2::VERSION) => {
            rpc::GetBestTipV2::query_payload(&mut bytes)?;
            dispatcher.push(P2pChannelsRpcAction::RequestReceived {
                peer_id,
                id,
                request: Box::new(P2pRpcRequest::BestTipWithProof),
            });
        }
        (rpc::AnswerSyncLedgerQueryV2::NAME, rpc::AnswerSyncLedgerQueryV2::VERSION) => {
            let (hash, query) = rpc::AnswerSyncLedgerQueryV2::query_payload(&mut bytes)?;
            let hash = v2::LedgerHash::from(v2::MinaBaseLedgerHash0StableV1(hash));

            dispatcher.push(P2pChannelsRpcAction::RequestReceived {
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

            dispatcher.push(P2pChannelsRpcAction::RequestReceived {
                peer_id,
                id,
                request,
            });
        }
        (rpc::GetTransitionChainV2::NAME, rpc::GetTransitionChainV2::VERSION) => {
            let hashes = rpc::GetTransitionChainV2::query_payload(&mut bytes)?;
            for hash in hashes {
                let hash = v2::StateHash::from(v2::DataHashLibStateHashStableV1(hash));

                dispatcher.push(P2pChannelsRpcAction::RequestReceived {
                    peer_id,
                    id,
                    request: Box::new(P2pRpcRequest::Block(hash)),
                });
            }
        }
        (rpc::GetSomeInitialPeersV1ForV2::NAME, rpc::GetSomeInitialPeersV1ForV2::VERSION) => {
            let () = rpc::GetSomeInitialPeersV1ForV2::query_payload(&mut bytes)?;
            dispatcher.push(P2pChannelsRpcAction::RequestReceived {
                peer_id,
                id,
                request: Box::new(P2pRpcRequest::InitialPeers),
            });
        }
        (name, version) => return Err(RpcQueryError::Unimplemented(name, version)),
    }
    Ok(())
}

fn dispatch_rpc_response<State, Action>(
    peer_id: PeerId,
    QueryHeader { tag, version, id }: &QueryHeader,
    mut bytes: &[u8],
    dispatcher: &mut Dispatcher<Action, State>,
) -> Result<(), RpcResponseError>
where
    State: crate::P2pStateTrait,
    Action: crate::P2pActionTrait<State>,
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

            dispatcher.push(P2pChannelsRpcAction::ResponseReceived {
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

            dispatcher.push(P2pChannelsRpcAction::ResponseReceived {
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

            dispatcher.push(P2pChannelsRpcAction::ResponseReceived {
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
                        dispatcher.push(P2pChannelsRpcAction::ResponseReceived {
                            peer_id,
                            id,
                            response,
                        });
                    }
                }
                _ => {
                    dispatcher.push(P2pChannelsRpcAction::ResponseReceived {
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
                dispatcher.push(P2pChannelsRpcAction::ResponseReceived {
                    peer_id,
                    id,
                    response: None,
                });
            } else {
                let peers = response
                    .into_iter()
                    .filter_map(P2pConnectionOutgoingInitOpts::try_from_mina_rpc)
                    .collect();
                dispatcher.push(P2pChannelsRpcAction::ResponseReceived {
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

#[derive(Debug, thiserror::Error)]
enum RpcQueryError<'a> {
    #[error(transparent)]
    Read(#[from] RpcQueryReadError),
    #[error("unimplemented rpc {}:{1}", String::from_utf8_lossy(.0))]
    Unimplemented(&'a [u8], Ver),
}

#[derive(Debug, thiserror::Error)]
enum RpcResponseError {
    #[error(transparent)]
    Read(#[from] RpcResponseReadError),
    #[error("rpc response {rpc_id} error: {error}")]
    Other { rpc_id: String, error: String },
}

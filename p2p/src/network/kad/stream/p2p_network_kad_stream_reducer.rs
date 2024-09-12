use openmina_core::{bug_condition, fuzzed_maybe, warn, Substate, SubstateAccess};
use quick_protobuf::{serialize_into_vec, BytesReader};
use redux::ActionWithMeta;

use crate::{
    stream::P2pNetworkKadOutgoingStreamError, Data, P2pLimits, P2pNetworkConnectionError,
    P2pNetworkKadState, P2pNetworkKademliaAction, P2pNetworkKademliaRpcReply,
    P2pNetworkKademliaRpcRequest, P2pNetworkSchedulerAction, P2pNetworkStreamProtobufError,
    P2pNetworkYamuxAction, P2pState, YamuxFlags,
};

use super::{
    super::Message, P2pNetworkKadIncomingStreamError, P2pNetworkKadIncomingStreamState,
    P2pNetworkKadOutgoingStreamState, P2pNetworkKadStreamState, P2pNetworkKademliaStreamAction,
};

impl P2pNetworkKadIncomingStreamState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, P2pNetworkKadState>,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: SubstateAccess<P2pNetworkKadState> + SubstateAccess<P2pState>,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let Some(P2pNetworkKadStreamState::Incoming(state)) = state_context
            .get_substate_mut()?
            .find_kad_stream_state_mut(action.peer_id(), action.stream_id())
        else {
            return Err("invalid stream".to_owned());
        };

        match (&state, action) {
            (
                P2pNetworkKadIncomingStreamState::Default,
                P2pNetworkKademliaStreamAction::New { incoming, .. },
            ) if *incoming => {
                *state = P2pNetworkKadIncomingStreamState::WaitingForRequest {
                    expect_close: false,
                };
                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::WaitingForRequest { .. },
                P2pNetworkKademliaStreamAction::IncomingData {
                    data,
                    addr,
                    peer_id,
                    stream_id,
                },
            ) => {
                let data = &data.0;
                let mut reader = BytesReader::from_bytes(data);

                match reader.read_varint32(data).map(|v| v as usize) {
                    Ok(encoded_len) if encoded_len > limits.kademlia_request() => {
                        *state = P2pNetworkKadIncomingStreamState::Error(
                            P2pNetworkStreamProtobufError::Limit(
                                encoded_len,
                                limits.kademlia_request(),
                            ),
                        );
                    }
                    Ok(encoded_len) => {
                        let remaining_len = reader.len();
                        if let Some(remaining_data) = data.get(data.len() - remaining_len..) {
                            if encoded_len > remaining_len {
                                *state = P2pNetworkKadIncomingStreamState::PartialRequestReceived {
                                    len: encoded_len,
                                    data: remaining_data.to_vec(),
                                };
                                return Ok(());
                            }

                            state.handle_incoming_request(encoded_len, remaining_data)?;
                        } else {
                            *state = P2pNetworkKadIncomingStreamState::Error(
                                P2pNetworkStreamProtobufError::Message("out of bounds".to_owned()),
                            );
                        }
                    }
                    Err(_) => {
                        *state = P2pNetworkKadIncomingStreamState::Error(
                            P2pNetworkStreamProtobufError::MessageLength,
                        );
                    }
                };

                let state = state.clone();
                let dispatcher = state_context.into_dispatcher();

                match state {
                    P2pNetworkKadIncomingStreamState::RequestIsReady {
                        data: P2pNetworkKademliaRpcRequest::FindNode { key },
                    } => {
                        // TODO: add callback
                        dispatcher.push(P2pNetworkKademliaStreamAction::WaitOutgoing {
                            addr: *addr,
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                        });
                        dispatcher.push(P2pNetworkKademliaAction::AnswerFindNodeRequest {
                            addr: *addr,
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                            key,
                        });
                    }
                    P2pNetworkKadIncomingStreamState::Error(error) => {
                        warn!(meta.time(); summary = "error handling kademlia action", error = display(&error));
                        dispatcher.push(P2pNetworkSchedulerAction::Error {
                            addr: *addr,
                            error: P2pNetworkConnectionError::from(
                                P2pNetworkKadIncomingStreamError::from(error),
                            ),
                        });
                    }
                    _ => bug_condition!("Invalid state"),
                }

                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::PartialRequestReceived { len, data },
                P2pNetworkKademliaStreamAction::IncomingData {
                    data: new_data,
                    addr,
                    peer_id,
                    stream_id,
                },
            ) => {
                let mut data = data.clone();
                data.extend_from_slice(&new_data.0);

                if *len > data.len() {
                    *state = P2pNetworkKadIncomingStreamState::PartialRequestReceived {
                        len: *len,
                        data,
                    };
                    return Ok(());
                }

                state.handle_incoming_request(*len, &data)?;
                let state = state.clone();
                let dispatcher = state_context.into_dispatcher();

                match state {
                    P2pNetworkKadIncomingStreamState::RequestIsReady {
                        data: P2pNetworkKademliaRpcRequest::FindNode { key },
                    } => {
                        // TODO: add callbacks
                        dispatcher.push(P2pNetworkKademliaStreamAction::WaitOutgoing {
                            addr: *addr,
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                        });
                        dispatcher.push(P2pNetworkKademliaAction::AnswerFindNodeRequest {
                            addr: *addr,
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                            key,
                        });
                    }
                    P2pNetworkKadIncomingStreamState::Error(error) => {
                        warn!(meta.time(); summary = "error handling kademlia action", error = display(&error));
                        dispatcher.push(P2pNetworkSchedulerAction::Error {
                            addr: *addr,
                            error: P2pNetworkConnectionError::from(
                                P2pNetworkKadIncomingStreamError::from(error),
                            ),
                        });
                    }
                    _ => bug_condition!("Invalid state"),
                }

                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::RequestIsReady { .. },
                P2pNetworkKademliaStreamAction::WaitOutgoing { .. },
            ) => {
                *state = P2pNetworkKadIncomingStreamState::WaitingForReply;
                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::WaitingForReply,
                P2pNetworkKademliaStreamAction::SendResponse {
                    data,
                    peer_id,
                    addr,
                    stream_id,
                },
            ) => {
                let message = Message::try_from(data).map_err(|e| e.to_string())?;
                let bytes = serialize_into_vec(&message).map_err(|e| format!("{e}"))?;
                *state = P2pNetworkKadIncomingStreamState::ResponseBytesAreReady {
                    bytes: bytes.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                let data = fuzzed_maybe!(bytes.clone().into(), crate::fuzzer::mutate_kad_data);
                let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

                dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                    addr: *addr,
                    stream_id: *stream_id,
                    data,
                    flags,
                });
                dispatcher.push(P2pNetworkKademliaStreamAction::WaitIncoming {
                    addr: *addr,
                    peer_id: *peer_id,
                    stream_id: *stream_id,
                });
                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::ResponseBytesAreReady { .. },
                P2pNetworkKademliaStreamAction::WaitIncoming { .. },
            ) => {
                *state = P2pNetworkKadIncomingStreamState::WaitingForRequest { expect_close: true };
                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::WaitingForRequest { expect_close, .. },
                P2pNetworkKademliaStreamAction::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                },
            ) if *expect_close => {
                *state = P2pNetworkKadIncomingStreamState::Closing;

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                    addr: *addr,
                    stream_id: *stream_id,
                    data: Data::empty(),
                    flags: YamuxFlags::FIN,
                });
                dispatcher.push(P2pNetworkKademliaStreamAction::Prune {
                    addr: *addr,
                    peer_id: *peer_id,
                    stream_id: *stream_id,
                });
                Ok(())
            }
            _ => Err(format!(
                "kademlia incoming stream state {state:?} is incorrect for action {action:?}",
            )),
        }
    }

    fn handle_incoming_request(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<Message>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkKadIncomingStreamState::Error(
                    P2pNetworkStreamProtobufError::Message(e.to_string()),
                );
                return Ok(());
            }
        };

        let data = match P2pNetworkKademliaRpcRequest::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkKadIncomingStreamState::Error(e.into());
                return Ok(());
            }
        };

        *self = P2pNetworkKadIncomingStreamState::RequestIsReady { data };
        Ok(())
    }
}

impl P2pNetworkKadOutgoingStreamState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, P2pNetworkKadState>,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: SubstateAccess<P2pNetworkKadState> + SubstateAccess<P2pState>,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let Some(P2pNetworkKadStreamState::Outgoing(state)) = state_context
            .get_substate_mut()?
            .find_kad_stream_state_mut(action.peer_id(), action.stream_id())
        else {
            bug_condition!("Stream not found");
            return Ok(());
        };

        match (&state, action) {
            (
                P2pNetworkKadOutgoingStreamState::Default,
                P2pNetworkKademliaStreamAction::New { incoming, .. },
            ) if !*incoming => {
                *state = P2pNetworkKadOutgoingStreamState::WaitingForRequest {
                    expect_close: false,
                };
                Ok(())
            }

            (
                P2pNetworkKadOutgoingStreamState::WaitingForRequest { .. },
                P2pNetworkKademliaStreamAction::SendRequest {
                    data,
                    addr,
                    stream_id,
                    peer_id,
                },
            ) => {
                let message = Message::from(data);
                let bytes = serialize_into_vec(&message).map_err(|e| format!("{e}"))?;
                *state = P2pNetworkKadOutgoingStreamState::RequestBytesAreReady {
                    bytes: bytes.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                let data = fuzzed_maybe!(bytes.clone().into(), crate::fuzzer::mutate_kad_data);
                let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

                dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                    addr: *addr,
                    stream_id: *stream_id,
                    data,
                    flags,
                });
                dispatcher.push(P2pNetworkKademliaStreamAction::WaitIncoming {
                    addr: *addr,
                    peer_id: *peer_id,
                    stream_id: *stream_id,
                });
                Ok(())
            }
            (
                P2pNetworkKadOutgoingStreamState::RequestBytesAreReady { .. },
                P2pNetworkKademliaStreamAction::WaitIncoming { .. },
            ) => {
                *state = P2pNetworkKadOutgoingStreamState::WaitingForReply;
                Ok(())
            }

            (
                P2pNetworkKadOutgoingStreamState::WaitingForReply { .. },
                P2pNetworkKademliaStreamAction::IncomingData {
                    data,
                    peer_id,
                    stream_id,
                    addr,
                },
            ) => {
                let data = &data.0;

                let mut reader = BytesReader::from_bytes(data);

                match reader.read_varint32(data).map(|v| v as usize) {
                    Ok(encoded_len) if encoded_len > limits.kademlia_response() => {
                        *state = P2pNetworkKadOutgoingStreamState::Error(
                            P2pNetworkStreamProtobufError::Limit(
                                encoded_len,
                                limits.kademlia_response(),
                            ),
                        );
                    }
                    Ok(encoded_len) => {
                        let remaining_len = reader.len();
                        if let Some(remaining_data) = data.get(data.len() - remaining_len..) {
                            if encoded_len > remaining_len {
                                *state = P2pNetworkKadOutgoingStreamState::PartialReplyReceived {
                                    len: encoded_len,
                                    data: remaining_data.to_vec(),
                                };
                                return Ok(());
                            }

                            state.handle_incoming_response(encoded_len, remaining_data)?;
                        } else {
                            *state = P2pNetworkKadOutgoingStreamState::Error(
                                P2pNetworkStreamProtobufError::Message("out of bounds".to_owned()),
                            );
                        }
                    }
                    Err(_) => {
                        *state = P2pNetworkKadOutgoingStreamState::Error(
                            P2pNetworkStreamProtobufError::MessageLength,
                        );
                    }
                };

                let state = state.clone();
                let dispatcher = state_context.into_dispatcher();

                match state {
                    P2pNetworkKadOutgoingStreamState::ResponseIsReady {
                        data:
                            P2pNetworkKademliaRpcReply::FindNode {
                                closer_peers: closest_peers,
                            },
                    } => {
                        dispatcher.push(P2pNetworkKademliaStreamAction::WaitOutgoing {
                            addr: *addr,
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                        });
                        dispatcher.push(P2pNetworkKademliaAction::UpdateFindNodeRequest {
                            addr: *addr,
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                            closest_peers,
                        });
                    }
                    P2pNetworkKadOutgoingStreamState::Error(error) => {
                        warn!(meta.time(); summary = "error handling kademlia action", error = display(&error));
                        dispatcher.push(P2pNetworkSchedulerAction::Error {
                            addr: *addr,
                            error: P2pNetworkConnectionError::from(
                                P2pNetworkKadOutgoingStreamError::from(error),
                            ),
                        });
                    }
                    _ => {
                        bug_condition!("Invalid state");
                    }
                }

                Ok(())
            }
            (
                P2pNetworkKadOutgoingStreamState::PartialReplyReceived { len, data },
                P2pNetworkKademliaStreamAction::IncomingData {
                    data: new_data,
                    stream_id,
                    peer_id,
                    addr,
                },
            ) => {
                let mut data = data.clone();
                data.extend_from_slice(&new_data.0);

                if *len > data.len() {
                    *state =
                        P2pNetworkKadOutgoingStreamState::PartialReplyReceived { len: *len, data };
                    return Ok(());
                }

                state.handle_incoming_response(*len, &data)?;
                let state = state.clone();
                let dispatcher = state_context.into_dispatcher();

                match state {
                    P2pNetworkKadOutgoingStreamState::ResponseIsReady {
                        data:
                            P2pNetworkKademliaRpcReply::FindNode {
                                closer_peers: closest_peers,
                            },
                    } => {
                        dispatcher.push(P2pNetworkKademliaStreamAction::WaitOutgoing {
                            addr: *addr,
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                        });
                        dispatcher.push(P2pNetworkKademliaAction::UpdateFindNodeRequest {
                            addr: *addr,
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                            closest_peers,
                        });
                    }
                    P2pNetworkKadOutgoingStreamState::Error(error) => {
                        warn!(meta.time(); summary = "error handling kademlia action", error = display(&error));
                        dispatcher.push(P2pNetworkSchedulerAction::Error {
                            addr: *addr,
                            error: P2pNetworkConnectionError::from(
                                P2pNetworkKadOutgoingStreamError::from(error),
                            ),
                        });
                    }
                    _ => bug_condition!("Invalid state"),
                }

                Ok(())
            }
            (
                P2pNetworkKadOutgoingStreamState::ResponseIsReady { .. },
                P2pNetworkKademliaStreamAction::WaitOutgoing { .. },
            ) => {
                *state = P2pNetworkKadOutgoingStreamState::WaitingForRequest { expect_close: true };
                Ok(())
            }
            (
                P2pNetworkKadOutgoingStreamState::WaitingForRequest { expect_close },
                P2pNetworkKademliaStreamAction::Close {
                    addr,
                    peer_id,
                    stream_id,
                },
            ) if *expect_close => {
                *state =
                    P2pNetworkKadOutgoingStreamState::RequestBytesAreReady { bytes: Vec::new() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                    addr: *addr,
                    stream_id: *stream_id,
                    data: Data::empty(),
                    flags: YamuxFlags::FIN,
                });
                dispatcher.push(P2pNetworkKademliaStreamAction::Prune {
                    addr: *addr,
                    peer_id: *peer_id,
                    stream_id: *stream_id,
                });
                Ok(())
            }
            (
                P2pNetworkKadOutgoingStreamState::Closing,
                P2pNetworkKademliaStreamAction::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                },
            ) => {
                *state = P2pNetworkKadOutgoingStreamState::Closed;
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkKademliaStreamAction::Prune {
                    addr: *addr,
                    peer_id: *peer_id,
                    stream_id: *stream_id,
                });
                Ok(())
            }
            _ => Err(format!(
                "kademlia outgoing stream state {state:?} is incorrect for action {action:?}",
            )),
        }
    }

    fn handle_incoming_response(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<Message>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkKadOutgoingStreamState::Error(
                    P2pNetworkStreamProtobufError::Message(e.to_string()),
                );
                return Ok(());
            }
        };

        let data = match P2pNetworkKademliaRpcReply::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkKadOutgoingStreamState::Error(e.into());
                return Ok(());
            }
        };

        *self = P2pNetworkKadOutgoingStreamState::ResponseIsReady { data };
        Ok(())
    }
}

impl P2pNetworkKadStreamState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, P2pNetworkKadState>,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: SubstateAccess<P2pNetworkKadState> + SubstateAccess<P2pState>,
        Action: crate::P2pActionTrait<State>,
    {
        let state = state_context.get_substate_mut()?;
        let action_ = action.action();
        let stream_state = match action_ {
            P2pNetworkKademliaStreamAction::New {
                peer_id,
                stream_id,
                incoming,
                ..
            } => state
                .create_kad_stream_state(*incoming, peer_id, stream_id)
                .map_err(|stream| {
                    format!("kademlia stream already exists for action {action:?}: {stream:?}")
                })?,
            P2pNetworkKademliaStreamAction::Prune {
                peer_id, stream_id, ..
            } => {
                return state
                    .remove_kad_stream_state(peer_id, stream_id)
                    .then_some(())
                    .ok_or_else(|| format!("kademlia stream not found for action {action:?}"))
            }
            _ => state
                .find_kad_stream_state(action_.peer_id(), action_.stream_id())
                .ok_or_else(|| format!("kademlia stream not found for action {action:?}"))?,
        };

        match stream_state {
            P2pNetworkKadStreamState::Incoming(_) => {
                P2pNetworkKadIncomingStreamState::reducer(state_context, action, limits)
            }
            P2pNetworkKadStreamState::Outgoing(_) => {
                P2pNetworkKadOutgoingStreamState::reducer(state_context, action, limits)
            }
        }
    }
}

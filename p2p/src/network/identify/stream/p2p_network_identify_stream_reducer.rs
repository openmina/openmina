use super::{
    P2pNetworkIdentifyStreamAction, P2pNetworkIdentifyStreamKind, P2pNetworkIdentifyStreamState,
};
use crate::{
    identify::P2pIdentifyAction,
    network::identify::{
        pb::Identify, stream::P2pNetworkIdentifyStreamError,
        stream_effectful::P2pNetworkIdentifyStreamEffectfulAction, P2pNetworkIdentify,
        P2pNetworkIdentifyState,
    },
    ConnectionAddr, Data, P2pLimits, P2pNetworkConnectionError, P2pNetworkSchedulerAction,
    P2pNetworkStreamProtobufError, P2pNetworkYamuxAction, PeerId, YamuxFlags,
};
use openmina_core::{bug_condition, warn, Substate, SubstateAccess};
use prost::Message;
use quick_protobuf::BytesReader;
use redux::{ActionWithMeta, Dispatcher};

impl P2pNetworkIdentifyStreamState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pNetworkIdentifyState>,
        action: ActionWithMeta<&P2pNetworkIdentifyStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: SubstateAccess<P2pNetworkIdentifyState>,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let substate = state_context.get_substate_mut()?;
        let stream_state = match action {
            P2pNetworkIdentifyStreamAction::New {
                peer_id, stream_id, ..
            } => substate
                .create_identify_stream_state(peer_id, stream_id)
                .map_err(|stream| {
                    format!("Identify stream already exists for action {action:?}: {stream:?}")
                })?,
            P2pNetworkIdentifyStreamAction::Prune {
                peer_id, stream_id, ..
            } => {
                return substate
                    .remove_identify_stream_state(peer_id, stream_id)
                    .then_some(())
                    .ok_or_else(|| format!("Identify stream not found for action {action:?}"));
            }
            a => substate
                .find_identify_stream_state_mut(a.peer_id(), a.stream_id())
                .ok_or_else(|| format!("Identify stream not found for action {action:?}"))?,
        };

        match &stream_state {
            P2pNetworkIdentifyStreamState::Default => {
                let P2pNetworkIdentifyStreamAction::New {
                    incoming,
                    addr,
                    peer_id,
                    stream_id,
                } = action
                else {
                    // enabling conditions should prevent receiving other actions in Default state
                    bug_condition!("Received action {:?} in Default state", action);
                    return Ok(());
                };

                let kind = P2pNetworkIdentifyStreamKind::from(*incoming);

                *stream_state = match kind {
                    // For incoming streams we prepare to send the Identify message
                    P2pNetworkIdentifyStreamKind::Incoming => {
                        P2pNetworkIdentifyStreamState::SendIdentify
                    }
                    // For outgoing streams we expect to get the Identify message from the remote peer
                    P2pNetworkIdentifyStreamKind::Outgoing => {
                        P2pNetworkIdentifyStreamState::RecvIdentify
                    }
                };

                if matches!(stream_state, P2pNetworkIdentifyStreamState::SendIdentify) {
                    let dispatcher = state_context.into_dispatcher();

                    dispatcher.push(P2pNetworkIdentifyStreamEffectfulAction::SendIdentify {
                        addr: *addr,
                        peer_id: *peer_id,
                        stream_id: *stream_id,
                    });
                }

                Ok(())
            }
            P2pNetworkIdentifyStreamState::RecvIdentify => match action {
                P2pNetworkIdentifyStreamAction::IncomingData {
                    data,
                    peer_id,
                    stream_id,
                    addr,
                } => {
                    let data = &data.0;
                    let mut reader = BytesReader::from_bytes(data);
                    let Ok(len) = reader.read_varint32(data).map(|v| v as usize) else {
                        *stream_state = P2pNetworkIdentifyStreamState::Error(
                            P2pNetworkStreamProtobufError::MessageLength,
                        );
                        return Ok(());
                    };

                    // TODO: implement as configuration option
                    if len > limits.identify_message() {
                        *stream_state = P2pNetworkIdentifyStreamState::Error(
                            P2pNetworkStreamProtobufError::Limit(len, limits.identify_message()),
                        );
                        return Ok(());
                    }

                    let data = &data[(data.len() - reader.len())..];

                    if len > reader.len() {
                        *stream_state = P2pNetworkIdentifyStreamState::IncomingPartialData {
                            len,
                            data: data.to_vec(),
                        };
                        Ok(())
                    } else {
                        stream_state.handle_incoming_identify_message(len, data)?;
                        let stream_state = stream_state.clone();
                        let dispatcher = state_context.into_dispatcher();

                        if let P2pNetworkIdentifyStreamState::IdentifyReceived { data } =
                            stream_state
                        {
                            dispatcher.push(P2pIdentifyAction::UpdatePeerInformation {
                                peer_id: *peer_id,
                                info: data,
                                addr: *addr,
                            });
                            dispatcher.push(P2pNetworkIdentifyStreamAction::Close {
                                addr: *addr,
                                peer_id: *peer_id,
                                stream_id: *stream_id,
                            });
                        } else {
                            let P2pNetworkIdentifyStreamState::Error(error) = stream_state else {
                                bug_condition!("Invalid stream state");
                                return Ok(());
                            };

                            warn!(meta.time(); summary = "error handling Identify action", error = display(&error));
                            dispatcher.push(P2pNetworkSchedulerAction::Error {
                                addr: *addr,
                                error: P2pNetworkConnectionError::IdentifyStreamError(
                                    P2pNetworkIdentifyStreamError::from(error),
                                ),
                            });
                        }
                        Ok(())
                    }
                }
                P2pNetworkIdentifyStreamAction::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                }
                | P2pNetworkIdentifyStreamAction::Close {
                    addr,
                    peer_id,
                    stream_id,
                } => {
                    let dispatcher = state_context.into_dispatcher();
                    Self::disconnect(dispatcher, *addr, *peer_id, *stream_id)
                }
                _ => {
                    // State and connection cleanup should be handled by timeout
                    bug_condition!("Received action {:?} in RecvIdentify state", action);
                    Ok(())
                }
            },
            P2pNetworkIdentifyStreamState::IncomingPartialData { len, data } => match action {
                P2pNetworkIdentifyStreamAction::IncomingData {
                    data: new_data,
                    peer_id,
                    addr,
                    stream_id,
                } => {
                    let mut data = data.clone();
                    data.extend_from_slice(&new_data.0);

                    if *len > data.len() {
                        *stream_state =
                            P2pNetworkIdentifyStreamState::IncomingPartialData { len: *len, data };
                        Ok(())
                    } else {
                        stream_state.handle_incoming_identify_message(*len, &data)?;

                        if let P2pNetworkIdentifyStreamState::IdentifyReceived { data } =
                            stream_state
                        {
                            let data = data.clone();
                            let dispatcher = state_context.into_dispatcher();
                            dispatcher.push(P2pIdentifyAction::UpdatePeerInformation {
                                peer_id: *peer_id,
                                info: data,
                                addr: *addr,
                            });
                            dispatcher.push(P2pNetworkIdentifyStreamAction::Close {
                                addr: *addr,
                                peer_id: *peer_id,
                                stream_id: *stream_id,
                            });
                        } else {
                            let P2pNetworkIdentifyStreamState::Error(error) = stream_state else {
                                bug_condition!("Invalid stream state");
                                return Ok(());
                            };

                            let error = error.clone();
                            let dispatcher = state_context.into_dispatcher();
                            warn!(meta.time(); summary = "error handling Identify action", error = display(&error));

                            dispatcher.push(P2pNetworkSchedulerAction::Error {
                                addr: *addr,
                                error: P2pNetworkConnectionError::IdentifyStreamError(
                                    P2pNetworkIdentifyStreamError::from(error),
                                ),
                            });
                        }

                        Ok(())
                    }
                }
                P2pNetworkIdentifyStreamAction::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                }
                | P2pNetworkIdentifyStreamAction::Close {
                    addr,
                    peer_id,
                    stream_id,
                } => {
                    let dispatcher = state_context.into_dispatcher();
                    Self::disconnect(dispatcher, *addr, *peer_id, *stream_id)
                }
                _ => {
                    // State and connection cleanup should be handled by timeout
                    bug_condition!("Received action {:?} in IncomingPartialData state", action);
                    Ok(())
                }
            },
            P2pNetworkIdentifyStreamState::SendIdentify => match action {
                P2pNetworkIdentifyStreamAction::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                }
                | P2pNetworkIdentifyStreamAction::Close {
                    addr,
                    peer_id,
                    stream_id,
                } => {
                    let dispatcher = state_context.into_dispatcher();
                    Self::disconnect(dispatcher, *addr, *peer_id, *stream_id)
                }
                _ => {
                    // State and connection cleanup should be handled by timeout
                    bug_condition!("Received action {:?} in SendIdentify state", action);
                    Ok(())
                }
            },
            P2pNetworkIdentifyStreamState::IdentifyReceived { .. } => match action {
                P2pNetworkIdentifyStreamAction::Close {
                    addr,
                    peer_id,
                    stream_id,
                }
                | P2pNetworkIdentifyStreamAction::RemoteClose {
                    addr,
                    peer_id,
                    stream_id,
                } => {
                    let dispatcher = state_context.into_dispatcher();
                    Self::disconnect(dispatcher, *addr, *peer_id, *stream_id)
                }
                _ => Ok(()),
            },
            P2pNetworkIdentifyStreamState::Error(_) => {
                // TODO
                Ok(())
            }
        }
    }

    fn handle_incoming_identify_message(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        let message = match Identify::decode(&data[..len]) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkIdentifyStreamState::Error(
                    P2pNetworkStreamProtobufError::Message(e.to_string()),
                );
                return Ok(());
            }
        };

        let data = match P2pNetworkIdentify::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkIdentifyStreamState::Error(e.into());
                return Ok(());
            }
        };

        *self = P2pNetworkIdentifyStreamState::IdentifyReceived {
            data: Box::new(data),
        };
        Ok(())
    }

    fn disconnect<Action, State>(
        dispatcher: &mut Dispatcher<Action, State>,
        addr: ConnectionAddr,
        peer_id: PeerId,
        stream_id: u32,
    ) -> Result<(), String>
    where
        State: SubstateAccess<P2pNetworkIdentifyState>,
        Action: crate::P2pActionTrait<State>,
    {
        dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
            addr,
            stream_id,
            data: Data::empty(),
            flags: YamuxFlags::FIN,
        });
        dispatcher.push(P2pNetworkIdentifyStreamAction::Prune {
            addr,
            peer_id,
            stream_id,
        });
        Ok(())
    }
}

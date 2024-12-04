use super::{
    P2pNetworkIdentifyStreamAction, P2pNetworkIdentifyStreamKind, P2pNetworkIdentifyStreamState,
};
use crate::{
    identify::P2pIdentifyAction,
    network::identify::{
        pb::{self, Identify},
        stream::P2pNetworkIdentifyStreamError,
        stream_effectful::P2pNetworkIdentifyStreamEffectfulAction,
        P2pNetworkIdentify, P2pNetworkIdentifyState,
    },
    token, ConnectionAddr, Data, P2pLimits, P2pNetworkConnectionError, P2pNetworkSchedulerAction,
    P2pNetworkStreamProtobufError, P2pNetworkYamuxAction, P2pState, PeerId, YamuxFlags,
};
use multiaddr::Multiaddr;
use openmina_core::{bug_condition, fuzzed_maybe, warn, Substate, SubstateAccess};
use prost::Message;
use quick_protobuf::BytesReader;
use redux::{ActionWithMeta, Dispatcher};

impl P2pNetworkIdentifyStreamState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pNetworkIdentifyState>,
        action: ActionWithMeta<P2pNetworkIdentifyStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let substate = state_context.get_substate_mut()?;
        let stream_state = match &action {
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
                .ok_or_else(|| format!("Identify stream not found for action {a:?}"))?,
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

                let kind = P2pNetworkIdentifyStreamKind::from(incoming);

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
                    let (dispatcher, state) = state_context.into_dispatcher_and_state();
                    let p2p_state: &P2pState = state.substate()?;

                    let addresses = p2p_state
                        .network
                        .scheduler
                        .listeners
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>();

                    dispatcher.push(
                        P2pNetworkIdentifyStreamEffectfulAction::GetListenAddresses {
                            addr,
                            peer_id,
                            stream_id,
                            addresses,
                        },
                    );
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
                                peer_id,
                                info: data,
                                addr,
                            });
                            dispatcher.push(P2pNetworkIdentifyStreamAction::Close {
                                addr,
                                peer_id,
                                stream_id,
                            });
                        } else {
                            let P2pNetworkIdentifyStreamState::Error(error) = stream_state else {
                                bug_condition!("Invalid stream state");
                                return Ok(());
                            };

                            warn!(meta.time(); summary = "error handling Identify action", error = display(&error));
                            dispatcher.push(P2pNetworkSchedulerAction::Error {
                                addr,
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
                    Self::disconnect(dispatcher, addr, peer_id, stream_id)
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
                    data.extend_from_slice(&new_data);

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
                                peer_id,
                                info: data,
                                addr,
                            });
                            dispatcher.push(P2pNetworkIdentifyStreamAction::Close {
                                addr,
                                peer_id,
                                stream_id,
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
                                addr,
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
                    Self::disconnect(dispatcher, addr, peer_id, stream_id)
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
                    Self::disconnect(dispatcher, addr, peer_id, stream_id)
                }
                P2pNetworkIdentifyStreamAction::SendIdentify {
                    addr,
                    peer_id,
                    stream_id,
                    addresses,
                } => {
                    let (dispatcher, state) = state_context.into_dispatcher_and_state();
                    let state = state.substate()?;
                    Self::send_identify(dispatcher, state, addr, peer_id, stream_id, addresses);
                    Ok(())
                }
                action => {
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
                    Self::disconnect(dispatcher, addr, peer_id, stream_id)
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

        let data = match P2pNetworkIdentify::try_from(message) {
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

    fn send_identify<Action, State>(
        dispatcher: &mut Dispatcher<Action, State>,
        state: &P2pState,
        addr: ConnectionAddr,
        peer_id: PeerId,
        stream_id: u32,
        mut listen_addrs: Vec<Multiaddr>,
    ) where
        Action: crate::P2pActionTrait<State>,
        State: crate::P2pStateTrait,
    {
        let config = &state.config;
        let ips = &config.external_addrs;
        let port = config.libp2p_port.unwrap_or(8302);

        listen_addrs.extend(
            ips.iter()
                .map(|ip| Multiaddr::from(*ip).with(multiaddr::Protocol::Tcp(port))),
        );

        let public_key = Some(state.config.identity_pub_key.clone());

        let mut protocols = vec![
            token::StreamKind::Identify(token::IdentifyAlgorithm::Identify1_0_0),
            token::StreamKind::Broadcast(token::BroadcastAlgorithm::Meshsub1_1_0),
            token::StreamKind::Rpc(token::RpcAlgorithm::Rpc0_0_1),
        ];
        if state.network.scheduler.discovery_state.is_some() {
            protocols.push(token::StreamKind::Discovery(
                token::DiscoveryAlgorithm::Kademlia1_0_0,
            ));
        }
        let identify_msg = P2pNetworkIdentify {
            protocol_version: Some("ipfs/0.1.0".to_string()),
            // TODO: include build info from GlobalConfig (?)
            agent_version: Some("openmina".to_owned()),
            public_key,
            listen_addrs,
            // TODO: other peers seem to report inaccurate information, should we implement this?
            observed_addr: None,
            protocols,
        };

        let mut out = Vec::new();
        let identify_msg_proto: pb::Identify = match (&identify_msg).try_into() {
            Ok(identify_msg_proto) => identify_msg_proto,
            Err(err) => {
                bug_condition!("error encoding message {:?}", err);
                return;
            }
        };

        if let Err(err) = prost::Message::encode_length_delimited(&identify_msg_proto, &mut out) {
            bug_condition!("error serializing message {:?}", err);
            return;
        }

        let data = fuzzed_maybe!(
            Data(out.into_boxed_slice()),
            crate::fuzzer::mutate_identify_msg
        );

        let flags = fuzzed_maybe!(Default::default(), crate::fuzzer::mutate_yamux_flags);

        dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
            addr,
            stream_id,
            data,
            flags,
        });

        dispatcher.push(P2pNetworkIdentifyStreamAction::Close {
            addr,
            peer_id,
            stream_id,
        });
    }
}

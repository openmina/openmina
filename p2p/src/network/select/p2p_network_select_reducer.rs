use openmina_core::{bug_condition, error, fuzz_maybe, fuzzed_maybe, Substate};
use redux::Timestamp;
use token::{
    AuthKind, DiscoveryAlgorithm, IdentifyAlgorithm, MuxKind, Protocol, RpcAlgorithm, StreamKind,
    Token,
};

use crate::{
    fuzzer::{mutate_select_authentication, mutate_select_multiplexing, mutate_select_stream},
    network::identify::P2pNetworkIdentifyStreamAction,
    ConnectionAddr, Data, P2pNetworkKademliaStreamAction, P2pNetworkNoiseAction,
    P2pNetworkPnetAction, P2pNetworkPubsubAction, P2pNetworkRpcAction, P2pNetworkSchedulerAction,
    P2pNetworkSchedulerState, P2pNetworkYamuxAction, P2pState, YamuxFlags,
};

use self::{p2p_network_select_state::P2pNetworkSelectStateInner, token::ParseTokenError};

use super::*;

impl P2pNetworkSelectState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, P2pNetworkSchedulerState>,
        action: redux::ActionWithMeta<&P2pNetworkSelectAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let select_kind = action.select_kind();

        let select_state = state_context
            .get_substate_mut()?
            .connection_state_mut(action.addr())
            .and_then(|conn| conn.select_state_mut(&select_kind))
            .ok_or_else(|| format!("Select state not found for {action:?}"))?;

        if let P2pNetworkSelectStateInner::Error(_) = &select_state.inner {
            return Ok(());
        }

        match action {
            // hack for noise
            P2pNetworkSelectAction::Init {
                incoming,
                addr,
                kind,
            } => {
                match (&select_state.inner, incoming) {
                    (P2pNetworkSelectStateInner::Initiator { .. }, true) => {
                        select_state.inner = P2pNetworkSelectStateInner::Responder
                    }
                    (P2pNetworkSelectStateInner::Responder, false) => {
                        select_state.inner = P2pNetworkSelectStateInner::Initiator {
                            proposing: token::Protocol::Mux(token::MuxKind::YamuxNoNewLine1_0_0),
                        }
                    }
                    _ => {}
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &P2pNetworkSchedulerState = state.substate()?;
                let state = state
                    .connection_state(addr)
                    .and_then(|state| state.select_state(kind))
                    .ok_or_else(|| format!("Select state not found for {action:?}"))?;

                if state.negotiated.is_none() && !incoming {
                    let mut tokens = vec![Token::Handshake];

                    match &state.inner {
                        P2pNetworkSelectStateInner::Uncertain { proposing } => {
                            tokens.push(Token::SimultaneousConnect);
                            tokens.push(Token::Protocol(*proposing));
                        }
                        P2pNetworkSelectStateInner::Initiator { proposing } => {
                            tokens.push(Token::Protocol(*proposing));
                        }
                        _ => {}
                    };
                    dispatcher.push(P2pNetworkSelectAction::OutgoingTokens {
                        addr: *addr,
                        kind: *kind,
                        tokens,
                    });
                }

                Ok(())
            }
            P2pNetworkSelectAction::IncomingData {
                data, addr, fin, ..
            }
            | P2pNetworkSelectAction::IncomingDataAuth { data, addr, fin }
            | P2pNetworkSelectAction::IncomingDataMux {
                data, addr, fin, ..
            } => {
                select_state.handle_incoming_data(data);

                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let state: &P2pNetworkSchedulerState = state.substate()?;
                let select_state = state
                    .connection_state(action.addr())
                    .and_then(|conn| conn.select_state(&select_kind))
                    .ok_or_else(|| format!("Select state not found for {action:?}"))?;

                if let P2pNetworkSelectStateInner::Error(error) = &select_state.inner {
                    dispatcher.push(P2pNetworkSchedulerAction::SelectError {
                        addr: *addr,
                        kind: select_kind,
                        error: error.to_owned(),
                    })
                } else {
                    for action in
                        select_state.forward_incoming_data(select_kind, *addr, data.clone(), *fin)
                    {
                        dispatcher.push(action)
                    }
                }

                Ok(())
            }
            P2pNetworkSelectAction::IncomingPayloadAuth { addr, fin, data }
            | P2pNetworkSelectAction::IncomingPayloadMux {
                addr, fin, data, ..
            }
            | P2pNetworkSelectAction::IncomingPayload {
                addr, fin, data, ..
            } => {
                select_state.recv.buffer.clear();

                P2pNetworkSelectState::handle_negotiated_token(
                    state_context,
                    select_kind,
                    addr,
                    data,
                    *fin,
                    meta.time(),
                )
            }
            P2pNetworkSelectAction::IncomingToken { kind, addr } => {
                let Some(token) = select_state.tokens.pop_front() else {
                    bug_condition!("Invalid state for action: {action:?}");
                    return Ok(());
                };
                select_state.handle_incoming_token(token, meta.time(), select_kind);

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let scheduler: &P2pNetworkSchedulerState = state.substate()?;
                let select_state = scheduler
                    .connection_state(addr)
                    .and_then(|stream| stream.select_state(kind))
                    .ok_or_else(|| format!("Select state not found for {action:?}"))?;

                if let P2pNetworkSelectStateInner::Error(error) = &select_state.inner {
                    dispatcher.push(P2pNetworkSchedulerAction::SelectError {
                        addr: *addr,
                        kind: *kind,
                        error: error.to_owned(),
                    });
                    return Ok(());
                }

                if let Some(token) = &select_state.to_send {
                    dispatcher.push(P2pNetworkSelectAction::OutgoingTokens {
                        addr: *addr,
                        kind: *kind,
                        tokens: vec![token.clone()],
                    })
                }

                if let Some(protocol) = select_state.negotiated {
                    let p2p_state: &P2pState = state.substate()?;

                    let expected_peer_id = p2p_state
                        .peer_with_connection(*addr)
                        .map(|(peer_id, _)| peer_id);

                    let incoming = matches!(
                        &select_state.inner,
                        P2pNetworkSelectStateInner::Responder { .. }
                    );
                    dispatcher.push(P2pNetworkSchedulerAction::SelectDone {
                        addr: *addr,
                        kind: *kind,
                        protocol,
                        incoming,
                        expected_peer_id,
                    })
                }

                Ok(())
            }
            P2pNetworkSelectAction::OutgoingTokens { addr, kind, tokens } => {
                let dispatcher = state_context.into_dispatcher();

                let mut data = {
                    let mut data = vec![];
                    for token in tokens {
                        data.extend_from_slice(token.name())
                    }
                    data.into()
                };

                let addr = *addr;
                match kind {
                    SelectKind::Authentication => {
                        fuzz_maybe!(&mut data, mutate_select_authentication);
                        dispatcher.push(P2pNetworkPnetAction::OutgoingData { addr, data });
                    }
                    SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                        fuzz_maybe!(&mut data, mutate_select_multiplexing);
                        dispatcher
                            .push(P2pNetworkNoiseAction::OutgoingDataSelectMux { addr, data });
                    }
                    SelectKind::Stream(_, stream_id) => {
                        if let Some(na) = tokens.iter().find(|t| t.name() == Token::Na.name()) {
                            dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                                addr,
                                stream_id: *stream_id,
                                data: na.name().to_vec().into(),
                                flags: YamuxFlags::FIN,
                            });
                        } else {
                            for token in tokens {
                                let data = fuzzed_maybe!(
                                    token.name().to_vec().into(),
                                    mutate_select_stream
                                );
                                dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                                    addr,
                                    stream_id: *stream_id,
                                    data,
                                    flags: Default::default(),
                                });
                            }
                        }
                    }
                }

                Ok(())
            }
            P2pNetworkSelectAction::Timeout { addr, kind } => {
                let error = "timeout".to_owned();
                select_state.inner = P2pNetworkSelectStateInner::Error(error.clone());
                openmina_core::warn!(meta.time(); "timeout");

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkSchedulerAction::SelectError {
                    addr: *addr,
                    kind: *kind,
                    error,
                });
                Ok(())
            }
        }
    }

    fn handle_incoming_data(&mut self, data: &Data) {
        if self.negotiated.is_none() {
            self.recv.put(data);
            loop {
                let parse_result = self.recv.parse_token();

                match parse_result {
                    Err(ParseTokenError) => {
                        self.inner = P2pNetworkSelectStateInner::Error("parse_token".to_owned());
                        self.recv.buffer.clear();
                        break;
                    }
                    Ok(None) => break,
                    Ok(Some(token)) => {
                        let done = matches!(
                            token,
                            token::Token::Protocol(..) | token::Token::UnknownProtocol(..)
                        );
                        self.tokens.push_back(token);

                        if done {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn handle_incoming_token(&mut self, token: Token, time: Timestamp, kind: SelectKind) {
        self.to_send = None;
        match &self.inner {
            P2pNetworkSelectStateInner::Error(_) => {}
            P2pNetworkSelectStateInner::Initiator { proposing } => match token {
                token::Token::Handshake => {}
                token::Token::Na => {
                    // TODO: check if we can propose alternative
                    self.inner = P2pNetworkSelectStateInner::Error("token is NA".to_owned());
                    self.negotiated = Some(None);
                }
                token::Token::SimultaneousConnect => {
                    // unexpected token
                    self.inner =
                        P2pNetworkSelectStateInner::Error("simultaneous connect token".to_owned());
                }
                token::Token::Protocol(response) => {
                    if response == *proposing {
                        self.negotiated = Some(Some(response));
                    } else {
                        self.inner = P2pNetworkSelectStateInner::Error(format!(
                            "protocol mismatch: {response:?} != {proposing:?}"
                        ));
                    }
                }
                token::Token::UnknownProtocol(name) => {
                    // unexpected token
                    self.inner = P2pNetworkSelectStateInner::Error(format!(
                        "unknown protocol `{}`",
                        String::from_utf8_lossy(&name)
                    ));
                    self.negotiated = Some(None);
                }
            },
            P2pNetworkSelectStateInner::Uncertain { proposing } => match token {
                token::Token::Handshake => {}
                token::Token::Na => {
                    let proposing = *proposing;
                    self.inner = P2pNetworkSelectStateInner::Initiator { proposing };
                }
                token::Token::SimultaneousConnect => {
                    // TODO: decide who is initiator
                }
                token::Token::Protocol(_) => {
                    self.inner = P2pNetworkSelectStateInner::Error(
                        "protocol mismatch: uncertain".to_owned(),
                    );
                }
                token::Token::UnknownProtocol(name) => {
                    self.inner = P2pNetworkSelectStateInner::Error(format!(
                        "protocol mismatch: uncertain with unknown protocol {}",
                        String::from_utf8_lossy(&name)
                    ));
                }
            },
            P2pNetworkSelectStateInner::Responder => match token {
                token::Token::Handshake => {
                    self.to_send = Some(token::Token::Handshake);
                }
                token::Token::Na => {}
                token::Token::SimultaneousConnect => {
                    self.to_send = Some(token::Token::Na);
                }
                token::Token::Protocol(protocol) => {
                    let reply = match protocol {
                        token::Protocol::Auth(_) => {
                            token::Token::Protocol(token::Protocol::Auth(token::AuthKind::Noise))
                        }
                        token::Protocol::Mux(token::MuxKind::Yamux1_0_0) => {
                            token::Token::Protocol(token::Protocol::Mux(token::MuxKind::Yamux1_0_0))
                        }
                        token::Protocol::Mux(token::MuxKind::YamuxNoNewLine1_0_0) => {
                            token::Token::Protocol(token::Protocol::Mux(
                                token::MuxKind::YamuxNoNewLine1_0_0,
                            ))
                        }
                        token::Protocol::Stream(
                            token::StreamKind::Rpc(_)
                            | token::StreamKind::Discovery(_)
                            | token::StreamKind::Broadcast(_)
                            | token::StreamKind::Identify(_)
                            | token::StreamKind::Ping(_)
                            | token::StreamKind::Bitswap(_)
                            | token::StreamKind::Status(_),
                        ) => token::Token::Protocol(protocol),
                    };
                    let negotiated = if let token::Token::Protocol(p) = &reply {
                        Some(*p)
                    } else {
                        None
                    };
                    self.negotiated = Some(negotiated);
                    self.to_send = Some(reply);
                }
                token::Token::UnknownProtocol(name) => {
                    const KNOWN_UNKNOWN_PROTOCOLS: [&str; 3] =
                        ["/ipfs/id/push/1.0.0", "/ipfs/id/1.0.0", "/mina/node-status"];
                    if !name.is_empty() {
                        if let Ok(str) = std::str::from_utf8(&name[1..]) {
                            let str = str.trim_end_matches('\n');
                            if !KNOWN_UNKNOWN_PROTOCOLS.iter().any(|s| (*s).eq(str)) {
                                self.inner = P2pNetworkSelectStateInner::Error(format!(
                                    "responder with unknown protocol {}",
                                    str
                                ));

                                openmina_core::error!(time; "unknown protocol: {str}, {kind:?}");
                            }
                        } else {
                            self.inner = P2pNetworkSelectStateInner::Error(format!(
                                "responder with invalid protocol data {:?}",
                                name
                            ));

                            openmina_core::error!(time; "invalid protocol: {name:?}, {kind:?}");
                        }
                    } else {
                        self.inner = P2pNetworkSelectStateInner::Error(
                            "responder with empty protocol".to_string(),
                        );

                        openmina_core::error!(time; "empty protocol: {kind:?}");
                    }
                    self.to_send = Some(token::Token::Na);
                    self.negotiated = Some(None);
                }
            },
        }
    }

    fn handle_negotiated_token<Action, State>(
        state_context: Substate<Action, State, P2pNetworkSchedulerState>,
        select_kind: SelectKind,
        addr: &ConnectionAddr,
        data: &Data,
        fin: bool,
        time: Timestamp,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (dispatcher, state) = state_context.into_dispatcher_and_state();
        let p2p_state: &P2pState = state.substate()?;
        let state: &P2pNetworkSchedulerState = state.substate()?;
        let state = state
            .connection_state(addr)
            .and_then(|state| state.select_state(&select_kind))
            .ok_or_else(|| "Select state not found incoming payload".to_owned())?;

        let Some(Some(negotiated)) = &state.negotiated else {
            bug_condition!(
                "Invalid negotiation state {:?} for incoming payload",
                state.negotiated,
            );
            return Ok(());
        };
        let addr = *addr;
        let data = data.clone();

        match negotiated {
            Protocol::Auth(AuthKind::Noise) => {
                dispatcher.push(P2pNetworkNoiseAction::IncomingData { addr, data });
            }
            Protocol::Mux(MuxKind::Yamux1_0_0 | MuxKind::YamuxNoNewLine1_0_0) => {
                dispatcher.push(P2pNetworkYamuxAction::IncomingData { addr, data });
            }
            Protocol::Stream(kind) => match select_kind {
                SelectKind::Stream(peer_id, stream_id) => match kind {
                    StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0) => {
                        if !fin {
                            dispatcher.push(P2pNetworkKademliaStreamAction::IncomingData {
                                addr,
                                peer_id,
                                stream_id,
                                data,
                            });
                        } else {
                            dispatcher.push(P2pNetworkKademliaStreamAction::RemoteClose {
                                addr,
                                peer_id,
                                stream_id,
                            });
                        }
                    }
                    StreamKind::Identify(IdentifyAlgorithm::Identify1_0_0) => {
                        if !fin {
                            dispatcher.push(P2pNetworkIdentifyStreamAction::IncomingData {
                                addr,
                                peer_id,
                                stream_id,
                                data,
                            });
                        } else {
                            dispatcher.push(P2pNetworkIdentifyStreamAction::RemoteClose {
                                addr,
                                peer_id,
                                stream_id,
                            });
                        }
                    }
                    StreamKind::Broadcast(_) => {
                        dispatcher.push(P2pNetworkPubsubAction::IncomingData {
                            peer_id,
                            addr,
                            stream_id,
                            data,
                            seen_limit: p2p_state.config.meshsub.mcache_len,
                        });
                    }
                    StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1) => {
                        dispatcher.push(P2pNetworkRpcAction::IncomingData {
                            addr,
                            peer_id,
                            stream_id,
                            data,
                        });
                    }
                    _ => error!(time;
                        "trying to negotiate unimplemented stream kind {kind:?}"
                    ),
                },
                _ => error!(time; "invalid select protocol kind: {:?}", kind),
            },
        }

        Ok(())
    }
}

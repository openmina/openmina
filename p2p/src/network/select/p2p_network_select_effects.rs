use crate::{network::identify::P2pNetworkIdentifyStreamAction, P2pNetworkPnetAction};

use super::{super::*, p2p_network_select_state::P2pNetworkSelectStateInner, *};

impl P2pNetworkSelectAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        use self::token::*;

        let Some(state) = store.state().network.scheduler.connections.get(self.addr()) else {
            return;
        };
        let state = match self.id() {
            SelectKind::Authentication => &state.select_auth,
            SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => &state.select_mux,
            SelectKind::Stream(_, stream_id) => match state.streams.get(stream_id) {
                Some(v) => &v.select,
                None => return,
            },
        };
        let (addr, kind) = (*self.addr(), *self.id());
        if let P2pNetworkSelectStateInner::Error(error) = &state.inner {
            store.dispatch(P2pNetworkSchedulerAction::SelectError {
                addr,
                kind,
                error: error.clone(),
            });
            return;
        }
        let report = if state.reported {
            None
        } else {
            state.negotiated
        };
        let incoming = matches!(&state.inner, P2pNetworkSelectStateInner::Responder { .. });
        match self {
            P2pNetworkSelectAction::Init {
                addr,
                kind,
                send_handshake,
                ..
            } => {
                if state.negotiated.is_none() {
                    let mut tokens = vec![];
                    if send_handshake {
                        tokens.push(Token::Handshake);
                    }
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
                    store.dispatch(P2pNetworkSelectAction::OutgoingTokens { addr, kind, tokens });
                }
            }
            P2pNetworkSelectAction::IncomingData {
                addr,
                kind,
                fin,
                data,
            } => {
                if matches!(&state.inner, P2pNetworkSelectStateInner::Error(..)) {
                    return;
                }

                if state.negotiated.is_some() {
                    store.dispatch(P2pNetworkSelectAction::IncomingPayload {
                        addr,
                        kind,
                        fin,
                        data: data.clone(),
                    });
                    return;
                }

                let payload_data = state.recv.buffer.clone();

                let mut tokens_parsed = false;
                let tokens = state.tokens.clone();

                for token in tokens {
                    if !tokens_parsed {
                        tokens_parsed = matches!(
                            token,
                            token::Token::Protocol(..) | token::Token::UnknownProtocol(..)
                        );
                    }

                    store.dispatch(P2pNetworkSelectAction::IncomingToken { addr, kind, token });
                }

                if tokens_parsed && !payload_data.is_empty() {
                    store.dispatch(P2pNetworkSelectAction::IncomingPayload {
                        addr,
                        kind,
                        fin,
                        data: payload_data.into(),
                    });
                }
            }
            P2pNetworkSelectAction::IncomingPayload {
                addr,
                kind: select_kind,
                fin,
                data,
            } => {
                if matches!(&state.inner, P2pNetworkSelectStateInner::Error(..)) {
                    return;
                }

                if let Some(Some(negotiated)) = &state.negotiated {
                    match negotiated {
                        Protocol::Auth(AuthKind::Noise) => {
                            store.dispatch(P2pNetworkNoiseAction::IncomingData {
                                addr,
                                data: data.clone(),
                            });
                        }
                        Protocol::Mux(MuxKind::Yamux1_0_0 | MuxKind::YamuxNoNewLine1_0_0) => {
                            store.dispatch(P2pNetworkYamuxAction::IncomingData {
                                addr,
                                data: data.clone(),
                            });
                        }
                        Protocol::Stream(kind) => match select_kind {
                            SelectKind::Stream(peer_id, stream_id) => {
                                match kind {
                                    StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0) => {
                                        if !fin {
                                            store.dispatch(
                                                P2pNetworkKademliaStreamAction::IncomingData {
                                                    addr,
                                                    peer_id,
                                                    stream_id,
                                                    data: data.clone(),
                                                },
                                            );
                                        } else {
                                            store.dispatch(
                                                P2pNetworkKademliaStreamAction::RemoteClose {
                                                    addr,
                                                    peer_id,
                                                    stream_id,
                                                },
                                            );
                                        }
                                    }
                                    StreamKind::Identify(IdentifyAlgorithm::Identify1_0_0) => {
                                        if !fin {
                                            //println!("==== {}", hex::encode(&a.data.0));
                                            store.dispatch(
                                                P2pNetworkIdentifyStreamAction::IncomingData {
                                                    addr,
                                                    peer_id,
                                                    stream_id,
                                                    data: data.clone(),
                                                },
                                            );
                                        } else {
                                            store.dispatch(
                                                P2pNetworkIdentifyStreamAction::RemoteClose {
                                                    addr,
                                                    peer_id,
                                                    stream_id,
                                                },
                                            );
                                        }
                                    }
                                    StreamKind::Identify(IdentifyAlgorithm::IdentifyPush1_0_0) => {
                                        //unimplemented!()
                                    }
                                    StreamKind::Broadcast(_) => {
                                        store.dispatch(P2pNetworkPubsubAction::IncomingData {
                                            peer_id,
                                            addr,
                                            stream_id,
                                            data,
                                        });
                                    }
                                    StreamKind::Ping(PingAlgorithm::Ping1_0_0) => {
                                        //unimplemented!()
                                    }
                                    StreamKind::Bitswap(_) => {
                                        //unimplemented!()
                                    }
                                    StreamKind::Status(_) => {
                                        //unimplemented!()
                                    }
                                    StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1) => {
                                        store.dispatch(P2pNetworkRpcAction::IncomingData {
                                            addr,
                                            peer_id,
                                            stream_id,
                                            data: data.clone(),
                                        });
                                    }
                                }
                            }
                            _ => {
                                openmina_core::error!(meta.time(); "invalid select protocol kind: {:?}", kind);
                            }
                        },
                    }
                } else {
                    unreachable!()
                }
            }
            P2pNetworkSelectAction::IncomingToken { addr, kind, .. } => {
                if let Some(token) = &state.to_send {
                    store.dispatch(P2pNetworkSelectAction::OutgoingTokens {
                        addr,
                        kind,
                        tokens: vec![token.clone()],
                    });
                }
            }
            P2pNetworkSelectAction::OutgoingTokens { addr, kind, tokens } => {
                let data = {
                    let mut data = vec![];
                    if tokens.is_empty() {
                        data.extend_from_slice(Token::Na.name());
                    } else {
                        for token in &tokens {
                            data.extend_from_slice(token.name())
                        }
                    }
                    data.into()
                };

                match &kind {
                    SelectKind::Authentication => {
                        store.dispatch(P2pNetworkPnetAction::OutgoingData { addr, data });
                    }
                    SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                        store.dispatch(P2pNetworkNoiseAction::OutgoingData { addr, data });
                    }
                    SelectKind::Stream(_, stream_id) => {
                        let flags = if tokens.is_empty() {
                            YamuxFlags::FIN
                        } else {
                            YamuxFlags::empty()
                        };
                        store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                            addr,
                            stream_id: *stream_id,
                            data,
                            flags,
                        });
                    }
                }
            }
            P2pNetworkSelectAction::Timeout { .. } => {}
        }
        if let Some(protocol) = report {
            store.dispatch(P2pNetworkSchedulerAction::SelectDone {
                addr,
                kind,
                protocol,
                incoming,
            });
        }
    }
}


use super::{super::*, p2p_network_select_state::P2pNetworkSelectStateInner, *};

impl P2pNetworkSelectAction {
    pub fn effects<Store, S>(&self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        use self::token::*;

        let Some(state) = store
            .state()
            .network
            .scheduler
            .connections
            .get(&self.addr())
        else {
            return;
        };
        let state = match self.id() {
            SelectKind::Authentication => &state.select_auth,
            SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => &state.select_mux,
            SelectKind::Stream(_, stream_id) => match state.streams.get(&stream_id) {
                Some(v) => &v.select,
                None => return,
            },
        };
        if let P2pNetworkSelectStateInner::Error(error) = &state.inner {
            store.dispatch(P2pNetworkSchedulerAction::SelectError {
                addr: self.addr(),
                kind: self.id(),
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
            Self::Init(a) => {
                if state.negotiated.is_none() {
                    let mut tokens = vec![];
                    if a.send_handshake {
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
                    store.dispatch(P2pNetworkSelectOutgoingTokensAction {
                        addr: a.addr,
                        kind: a.kind,
                        tokens,
                    });
                }
            }
            Self::IncomingData(a) => {
                if let Some(Some(negotiated)) = &state.negotiated {
                    match negotiated {
                        Protocol::Auth(AuthKind::Noise) => {
                            store.dispatch(P2pNetworkNoiseAction::IncomingData {
                                addr: a.addr,
                                data: a.data.clone(),
                            });
                        }
                        Protocol::Mux(MuxKind::Yamux1_0_0 | MuxKind::YamuxNoNewLine1_0_0) => {
                            store.dispatch(P2pNetworkYamuxIncomingDataAction {
                                addr: a.addr,
                                data: a.data.clone(),
                            });
                        }
                        Protocol::Stream(kind) => match a.kind {
                            SelectKind::Stream(peer_id, stream_id) => {
                                match kind {
                                    StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0) => {
                                        if !a.fin {
                                            store.dispatch(
                                                P2pNetworkKademliaStreamAction::IncomingData {
                                                    addr: a.addr,
                                                    peer_id,
                                                    stream_id,
                                                    data: a.data.clone(),
                                                },
                                            );
                                        } else {
                                            store.dispatch(
                                                P2pNetworkKademliaStreamAction::RemoteClose {
                                                    addr: a.addr,
                                                    peer_id,
                                                    stream_id,
                                                },
                                            );
                                        }
                                    }
                                    StreamKind::Broadcast(BroadcastAlgorithm::Meshsub1_1_0) => {
                                        // send to meshsub handler
                                        unimplemented!()
                                    }
                                    StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1) => {
                                        store.dispatch(P2pNetworkRpcIncomingDataAction {
                                            addr: a.addr,
                                            peer_id,
                                            stream_id,
                                            data: a.data.clone(),
                                        });
                                    }
                                }
                            }
                            _ => {
                                openmina_core::error!(meta.time(); "invalid select protocol kind: {:?}", a.kind);
                            }
                        },
                    }
                } else {
                    let tokens = state.tokens.clone();
                    for token in tokens {
                        store.dispatch(P2pNetworkSelectIncomingTokenAction {
                            addr: a.addr,
                            kind: a.kind,
                            token,
                        });
                    }
                }
            }
            Self::IncomingToken(a) => {
                if let Some(token) = &state.to_send {
                    store.dispatch(P2pNetworkSelectOutgoingTokensAction {
                        addr: a.addr,
                        kind: a.kind,
                        tokens: vec![token.clone()],
                    });
                }
            }
            Self::OutgoingTokens(a) => {
                let mut data = vec![];
                for token in &a.tokens {
                    data.extend_from_slice(token.name())
                }
                match &a.kind {
                    SelectKind::Authentication => {
                        store.dispatch(P2pNetworkPnetAction::OutgoingData {
                            addr: a.addr,
                            data: data.into(),
                        });
                    }
                    SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                        store.dispatch(P2pNetworkNoiseAction::OutgoingData {
                            addr: a.addr,
                            data: data.into(),
                        });
                    }
                    SelectKind::Stream(_, stream_id) => {
                        for token in &a.tokens {
                            store.dispatch(P2pNetworkYamuxOutgoingDataAction {
                                addr: a.addr,
                                stream_id: *stream_id,
                                data: token.name().to_vec().into(),
                                fin: matches!(token, &token::Token::Na),
                            });
                        }
                    }
                }
            }
        }
        if let Some(protocol) = report {
            store.dispatch(P2pNetworkSchedulerAction::SelectDone {
                addr: self.addr(),
                kind: self.id(),
                protocol,
                incoming,
            });
        }
    }
}

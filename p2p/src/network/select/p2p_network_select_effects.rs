use self::token::{
    AuthKind, DiscoveryAlgorithm, IdentifyAlgorithm, MuxKind, Protocol, RpcAlgorithm, StreamKind,
    Token,
};

use openmina_core::{bug_condition, fuzz_maybe, fuzzed_maybe};

use crate::{
    fuzzer::{mutate_select_authentication, mutate_select_multiplexing, mutate_select_stream},
    network::identify::P2pNetworkIdentifyStreamAction,
    P2pNetworkPnetAction,
};

use super::{super::*, p2p_network_select_state::P2pNetworkSelectStateInner, *};

impl P2pNetworkSelectAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let Some(state) = store.state().network.scheduler.connections.get(self.addr()) else {
            return;
        };
        let state = match self.select_kind() {
            SelectKind::Authentication => &state.select_auth,
            SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => &state.select_mux,
            SelectKind::Stream(_, stream_id) => match state.streams.get(&stream_id) {
                Some(v) => &v.select,
                None => return,
            },
        };

        let (addr, select_kind) = (*self.addr(), self.select_kind());
        if let P2pNetworkSelectStateInner::Error(error) = &state.inner {
            store.dispatch(P2pNetworkSchedulerAction::SelectError {
                addr,
                kind: select_kind,
                error: error.clone(),
            });
            return;
        }
        let report = state.negotiated;
        let incoming = matches!(&state.inner, P2pNetworkSelectStateInner::Responder { .. });
        match self {
            P2pNetworkSelectAction::Init { addr, kind, .. } => {
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
                    store.dispatch(P2pNetworkSelectAction::OutgoingTokens { addr, kind, tokens });
                }
            }
            P2pNetworkSelectAction::IncomingDataAuth { addr, fin, data } => {
                let kind = SelectKind::Authentication;
                for action in state.forward_incoming_data(kind, addr, data, fin) {
                    store.dispatch(action);
                }
            }
            P2pNetworkSelectAction::IncomingDataMux {
                addr,
                peer_id,
                fin,
                data,
            } => {
                let kind = match peer_id {
                    Some(peer_id) => SelectKind::Multiplexing(peer_id),
                    None => SelectKind::MultiplexingNoPeerId,
                };
                for action in state.forward_incoming_data(kind, addr, data, fin) {
                    store.dispatch(action);
                }
            }
            P2pNetworkSelectAction::IncomingData {
                addr,
                peer_id,
                stream_id,
                fin,
                data,
            } => {
                let kind = SelectKind::Stream(peer_id, stream_id);
                for action in state.forward_incoming_data(kind, addr, data, fin) {
                    store.dispatch(action);
                }
            }
            P2pNetworkSelectAction::IncomingPayloadAuth {
                addr,
                fin,
                ref data,
                ..
            }
            | P2pNetworkSelectAction::IncomingPayloadMux {
                addr,
                fin,
                ref data,
                ..
            }
            | P2pNetworkSelectAction::IncomingPayload {
                addr,
                fin,
                ref data,
                ..
            } => {
                let Some(Some(negotiated)) = &state.negotiated else {
                    bug_condition!(
                        "Invalid negotiation state {:?} for action {:?}",
                        state.negotiated,
                        self
                    );
                    return;
                };
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
                                StreamKind::Broadcast(_) => {
                                    store.dispatch(P2pNetworkPubsubAction::IncomingData {
                                        peer_id,
                                        addr,
                                        stream_id,
                                        data: data.clone(),
                                        seen_limit: store.state().config.meshsub.mcache_len,
                                    });
                                }
                                StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1) => {
                                    store.dispatch(P2pNetworkRpcAction::IncomingData {
                                        addr,
                                        peer_id,
                                        stream_id,
                                        data: data.clone(),
                                    });
                                }
                                _ => {
                                    bug_condition!(
                                        "trying to negotiate unimplemented stream kind {:?}",
                                        kind
                                    );
                                }
                            }
                        }
                        _ => {
                            openmina_core::error!(meta.time(); "invalid select protocol kind: {:?}", kind);
                        }
                    },
                }
            }
            P2pNetworkSelectAction::IncomingToken { addr, kind, .. } => {
                if let Some(token) = &state.to_send {
                    let tokens = vec![token.clone()];
                    store.dispatch(P2pNetworkSelectAction::OutgoingTokens { addr, kind, tokens });
                }

                if let Some(protocol) = report {
                    let expected_peer_id = store
                        .state()
                        .peer_with_connection(addr)
                        .map(|(peer_id, _)| peer_id);

                    store.dispatch(P2pNetworkSchedulerAction::SelectDone {
                        addr,
                        kind: select_kind,
                        protocol,
                        incoming,
                        expected_peer_id,
                    });
                }
            }
            P2pNetworkSelectAction::OutgoingTokens { addr, kind, tokens } => {
                let mut data = {
                    let mut data = vec![];
                    for token in &tokens {
                        data.extend_from_slice(token.name())
                    }
                    data.into()
                };

                match &kind {
                    SelectKind::Authentication => {
                        fuzz_maybe!(&mut data, mutate_select_authentication);
                        store.dispatch(P2pNetworkPnetAction::OutgoingData { addr, data });
                    }
                    SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                        fuzz_maybe!(&mut data, mutate_select_multiplexing);
                        store.dispatch(P2pNetworkNoiseAction::OutgoingDataSelectMux { addr, data });
                    }
                    SelectKind::Stream(_, stream_id) => {
                        if let Some(na) = tokens.iter().find(|t| t.name() == Token::Na.name()) {
                            store.dispatch(P2pNetworkYamuxAction::OutgoingData {
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
                                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                                    addr,
                                    stream_id: *stream_id,
                                    data,
                                    flags: Default::default(),
                                });
                            }
                        }
                    }
                }
            }
            P2pNetworkSelectAction::Timeout { .. } => {}
        }
    }
}

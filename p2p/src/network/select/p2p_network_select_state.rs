use std::collections::VecDeque;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::P2pNetworkPnetOutgoingDataAction;

use super::{super::*, *};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectState {
    pub recv: token::State,
    pub tokens: VecDeque<token::Token>,

    pub negotiated: Option<Option<token::Protocol>>,
    pub reported: bool,

    pub inner: P2pNetworkSelectStateInner,
    pub to_send: Option<token::Token>,
}

impl P2pNetworkSelectState {
    pub fn initiator_auth(kind: token::AuthKind) -> Self {
        P2pNetworkSelectState {
            inner: P2pNetworkSelectStateInner::Uncertain {
                proposing: token::Protocol::Auth(kind),
            },
            ..Default::default()
        }
    }

    pub fn initiator_mux(kind: token::MuxKind) -> Self {
        P2pNetworkSelectState {
            inner: P2pNetworkSelectStateInner::Initiator {
                proposing: token::Protocol::Mux(kind),
            },
            ..Default::default()
        }
    }

    pub fn initiator_stream(kind: token::StreamKind) -> Self {
        P2pNetworkSelectState {
            inner: P2pNetworkSelectStateInner::Initiator {
                proposing: token::Protocol::Stream(kind),
            },
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkSelectStateInner {
    Error(String),
    Initiator { proposing: token::Protocol },
    Uncertain { proposing: token::Protocol },
    Responder,
}

impl Default for P2pNetworkSelectStateInner {
    fn default() -> Self {
        Self::Responder
    }
}

impl P2pNetworkSelectAction {
    pub fn effects<Store, S>(&self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingTokenAction: redux::EnablingCondition<S>,
        P2pNetworkSchedulerSelectErrorAction: redux::EnablingCondition<S>,
        P2pNetworkSchedulerSelectDoneAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectOutgoingTokensAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkRpcIncomingDataAction: redux::EnablingCondition<S>,
        // P2pNetworkKademliaAction: redux::EnablingCondition<S>,
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
            store.dispatch(P2pNetworkSchedulerSelectErrorAction {
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
            Self::IncomingData(a) => {
                if let Some(Some(negotiated)) = &state.negotiated {
                    match negotiated {
                        Protocol::Auth(AuthKind::Noise) => {
                            store.dispatch(P2pNetworkNoiseIncomingDataAction {
                                addr: a.addr,
                                data: a.data.clone(),
                            });
                        }
                        Protocol::Mux(MuxKind::Yamux1_0_0) => {
                            store.dispatch(P2pNetworkYamuxIncomingDataAction {
                                addr: a.addr,
                                data: a.data.clone(),
                            });
                        }
                        Protocol::Stream(kind) => match a.kind {
                            SelectKind::Stream(peer_id, stream_id) => {
                                match kind {
                                    StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0) => {
                                        // send to kademlia handler
                                        unimplemented!()
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
                        store.dispatch(P2pNetworkPnetOutgoingDataAction {
                            addr: a.addr,
                            data: data.into(),
                        });
                    }
                    SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                        store.dispatch(P2pNetworkNoiseOutgoingDataAction {
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
            store.dispatch(P2pNetworkSchedulerSelectDoneAction {
                addr: self.addr(),
                kind: self.id(),
                protocol,
                incoming,
            });
        }
    }
}

impl P2pNetworkSelectState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkSelectAction>) {
        if let P2pNetworkSelectStateInner::Error(_) = &self.inner {
            return;
        }

        if self.negotiated.is_some() {
            self.reported = true;
        }

        let (action, _meta) = action.split();
        match action {
            // hack for noise
            P2pNetworkSelectAction::Init(a) => match (&self.inner, a.incoming) {
                (P2pNetworkSelectStateInner::Initiator { .. }, true) => {
                    self.inner = P2pNetworkSelectStateInner::Responder
                }
                (P2pNetworkSelectStateInner::Responder, false) => {
                    self.inner = P2pNetworkSelectStateInner::Initiator {
                        proposing: token::Protocol::Mux(token::MuxKind::Yamux1_0_0),
                    }
                }
                _ => {}
            },
            P2pNetworkSelectAction::IncomingData(a) => {
                if self.negotiated.is_none() {
                    self.recv.put(&a.data);
                    loop {
                        match self.recv.parse_token() {
                            Err(()) => {
                                self.inner =
                                    P2pNetworkSelectStateInner::Error("parse_token".to_owned());
                                break;
                            }
                            Ok(None) => break,
                            Ok(Some(token)) => self.tokens.push_back(token),
                        }
                    }
                }
            }
            P2pNetworkSelectAction::IncomingToken(_) => {
                let Some(token) = self.tokens.pop_front() else {
                    return;
                };
                self.to_send = None;
                match &self.inner {
                    P2pNetworkSelectStateInner::Error(_) => {}
                    P2pNetworkSelectStateInner::Initiator { proposing } => match token {
                        token::Token::Handshake => {}
                        token::Token::Na => {
                            // TODO: check if we can propose alternative
                            self.inner =
                                P2pNetworkSelectStateInner::Error("token is NA".to_owned());
                            self.negotiated = Some(None);
                        }
                        token::Token::SimultaneousConnect => {
                            // unexpected token
                            self.inner = P2pNetworkSelectStateInner::Error(
                                "simultaneous connect token".to_owned(),
                            );
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
                        token::Token::Handshake => {}
                        token::Token::Na => {}
                        token::Token::SimultaneousConnect => {
                            self.to_send = Some(token::Token::Na);
                        }
                        token::Token::Protocol(protocol) => {
                            let reply = match protocol {
                                token::Protocol::Auth(_) => token::Token::Protocol(
                                    token::Protocol::Auth(token::AuthKind::Noise),
                                ),
                                token::Protocol::Mux(_) => token::Token::Protocol(
                                    token::Protocol::Mux(token::MuxKind::Yamux1_0_0),
                                ),
                                token::Protocol::Stream(token::StreamKind::Rpc(_)) => {
                                    token::Token::Protocol(protocol)
                                }
                                token::Protocol::Stream(token::StreamKind::Broadcast(_)) => {
                                    token::Token::Na
                                }
                                token::Protocol::Stream(token::StreamKind::Discovery(_)) => {
                                    token::Token::Na
                                }
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
                            openmina_core::error!(_meta.time(); "unknown protocol: {}", String::from_utf8_lossy(&name));
                            self.to_send = Some(token::Token::Na);
                            self.negotiated = Some(None);
                        }
                    },
                }
            }
            P2pNetworkSelectAction::OutgoingTokens(_) => {}
        }
    }
}

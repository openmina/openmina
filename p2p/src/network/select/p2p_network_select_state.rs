use std::collections::VecDeque;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{P2pMioService, P2pNetworkPnetOutgoingDataAction};

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectState {
    pub recv: token::State,
    pub tokens: VecDeque<token::Token>,

    pub negotiated: Option<token::Protocol>,

    pub inner: P2pNetworkSelectStateInner,
    pub to_send: Option<token::Token>,
}

impl P2pNetworkSelectState {
    pub fn initiator_auth(kind: token::AuthKind) -> Self {
        P2pNetworkSelectState {
            inner: P2pNetworkSelectStateInner::Initiator {
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
    Error,
    Initiator { proposing: token::Protocol },
    Uncertain { proposing: token::Protocol },
    Responder { proposing: Option<token::Protocol> },
}

impl Default for P2pNetworkSelectStateInner {
    fn default() -> Self {
        Self::Responder { proposing: None }
    }
}

impl P2pNetworkSelectAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingTokenAction: redux::EnablingCondition<S>,
    {
        let kind = self.select_kind();
        let Some(state) = store
            .state()
            .network
            .connection
            .connections
            .get(&self.addr())
        else {
            return;
        };
        let state = match kind {
            SelectKind::Authentication => &state.select_auth,
            SelectKind::Multiplexing => &state.select_mux,
            SelectKind::Stream => match self.stream_id().and_then(|s| state.streams.get(&s)) {
                Some(v) => v,
                None => return,
            },
        };
        if let P2pNetworkSelectStateInner::Error = &state.inner {
            return;
        }
        match self {
            Self::Init(a) => {
                let mut data = token::Token::Handshake.name().to_vec();
                if let P2pNetworkSelectStateInner::Uncertain { proposing } = &state.inner {
                    data.extend_from_slice(token::Token::SimultaneousConnect.name());
                    data.extend_from_slice(token::Token::Protocol(*proposing).name());
                }
                store.dispatch(P2pNetworkPnetOutgoingDataAction {
                    addr: a.addr,
                    data: data.into_boxed_slice(),
                });
            }
            Self::IncomingData(a) => {
                let tokens = state.tokens.clone();
                for token in tokens {
                    store.dispatch(P2pNetworkSelectIncomingTokenAction {
                        addr: a.addr,
                        peer_id: a.peer_id,
                        stream_id: a.stream_id,
                        token,
                    });
                }
            }
            Self::IncomingToken(a) => {
                if let Some(token) = &state.to_send {
                    store.dispatch(P2pNetworkPnetOutgoingDataAction {
                        addr: a.addr,
                        data: token.name().to_vec().into_boxed_slice(),
                    });
                }
            }
        }
    }
}

impl P2pNetworkSelectState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkSelectAction>) {
        if let P2pNetworkSelectStateInner::Error = &self.inner {
            return;
        }

        let (action, _meta) = action.split();
        match action {
            P2pNetworkSelectAction::Init(a) => {
                // TODO: implement select for stream
                let proposing = match action.select_kind() {
                    SelectKind::Authentication => token::Protocol::Auth(token::AuthKind::Noise),
                    SelectKind::Multiplexing => token::Protocol::Mux(token::MuxKind::Yamux1_0_0),
                    SelectKind::Stream => {
                        unimplemented!()
                    }
                };
                self.inner = if a.incoming {
                    P2pNetworkSelectStateInner::Responder {
                        proposing: Some(proposing),
                    }
                } else {
                    P2pNetworkSelectStateInner::Uncertain { proposing }
                };
            }
            P2pNetworkSelectAction::IncomingData(a) => {
                if let Some(negotiated) = &self.negotiated {
                    match negotiated {
                        token::Protocol::Auth(token::AuthKind::Noise) => {
                            //
                            dbg!(a.data.len());
                        }
                        token::Protocol::Mux(token::MuxKind::Yamux1_0_0) => {
                            // TODO:
                            unimplemented!()
                        }
                        token::Protocol::Stream(_) => unimplemented!(),
                    }
                } else {
                    self.recv.put(&a.data);
                    loop {
                        match self.recv.parse_token() {
                            Err(()) => {
                                self.inner = P2pNetworkSelectStateInner::Error;
                                break;
                            }
                            Ok(None) => break,
                            Ok(Some(token)) => self.tokens.push_back(token),
                        }
                    }
                }
            }
            P2pNetworkSelectAction::IncomingToken(_) => {
                let Some(token) = dbg!(self.tokens.pop_front()) else {
                    return;
                };
                self.to_send = None;
                match &self.inner {
                    P2pNetworkSelectStateInner::Error => {}
                    P2pNetworkSelectStateInner::Initiator { proposing } => match token {
                        token::Token::Handshake => {}
                        token::Token::Na => {
                            // TODO: check if we can propose alternative
                            self.inner = P2pNetworkSelectStateInner::Error;
                        }
                        token::Token::SimultaneousConnect => {
                            // unexpected token
                            self.inner = P2pNetworkSelectStateInner::Error;
                        }
                        token::Token::Protocol(response) => {
                            if response == *proposing {
                                self.to_send = Some(token::Token::Protocol(response));
                                self.negotiated = Some(response);
                            } else {
                                self.inner = P2pNetworkSelectStateInner::Error;
                            }
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
                            self.inner = P2pNetworkSelectStateInner::Error;
                        }
                    },
                    P2pNetworkSelectStateInner::Responder { proposing } => match token {
                        token::Token::Handshake => {}
                        token::Token::Na => {}
                        token::Token::SimultaneousConnect => {
                            self.to_send = Some(token::Token::Na);
                        }
                        token::Token::Protocol(response) => {
                            // TODO: check if we have the protocol
                            let _ = proposing;
                            self.to_send = Some(token::Token::Protocol(response));
                            self.negotiated = Some(response);
                        }
                    },
                }
            }
        }
    }
}

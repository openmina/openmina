use std::collections::VecDeque;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{P2pMioService, P2pNetworkPnetOutgoingDataAction};

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectState {
    pub kind: SelectKind,

    pub recv: token::State,
    pub tokens: VecDeque<token::Token>,

    pub negotiated: Option<token::Protocol>,

    pub inner: P2pNetworkSelectStateInner,
    pub to_send: Option<token::Token>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum P2pNetworkSelectStateInner {
    #[default]
    Responder,
    Initiator,
    Simultaneous,
    Error,
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
            Self::Init(a) => match kind {
                SelectKind::Authentication => {
                    let mut data = token::Token::Handshake.name().to_vec();
                    if let P2pNetworkSelectStateInner::Initiator = &state.inner {
                        data.extend_from_slice(token::Token::SimultaneousConnect.name());
                        data.extend_from_slice(token::AuthKind::Noise.name());
                    }
                    store.dispatch(P2pNetworkPnetOutgoingDataAction {
                        addr: a.addr,
                        data: data.into_boxed_slice(),
                    });
                }
                _ => unimplemented!(),
            },
            Self::IncomingData(a) => {
                let tokens = state
                    .tokens
                    .iter()
                    .map(|token| token.clone())
                    .collect::<Vec<_>>();
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
                self.kind = action.select_kind();
                self.inner = if a.incoming {
                    P2pNetworkSelectStateInner::Responder
                } else {
                    P2pNetworkSelectStateInner::Initiator
                };
            }
            P2pNetworkSelectAction::IncomingData(a) => {
                if let Some(negotiated) = &self.negotiated {
                    // TODO: send to the negotiated handler
                    let _ = negotiated;
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
                let Some(token) = self.tokens.pop_front() else {
                    return;
                };
                match dbg!(token) {
                    token::Token::Handshake => {}
                    _ => {}
                }
            }
        }
    }
}

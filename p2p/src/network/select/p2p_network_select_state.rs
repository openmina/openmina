use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::*;

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

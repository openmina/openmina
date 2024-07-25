use std::collections::VecDeque;

use redux::Timestamp;
use serde::{Deserialize, Serialize};
use token::Token;

use crate::{ConnectionAddr, Data, P2pTimeouts};

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectState {
    pub time: Option<Timestamp>,
    pub recv: token::State,
    pub tokens: VecDeque<token::Token>,

    pub negotiated: Option<Option<token::Protocol>>,

    pub inner: P2pNetworkSelectStateInner,
    pub to_send: Option<token::Token>,
}

impl P2pNetworkSelectState {
    pub fn initiator_auth(kind: token::AuthKind, time: Timestamp) -> Self {
        P2pNetworkSelectState {
            time: Some(time),
            inner: P2pNetworkSelectStateInner::Uncertain {
                proposing: token::Protocol::Auth(kind),
            },
            ..Default::default()
        }
    }

    pub fn initiator_mux(kind: token::MuxKind, time: Timestamp) -> Self {
        P2pNetworkSelectState {
            time: Some(time),
            inner: P2pNetworkSelectStateInner::Initiator {
                proposing: token::Protocol::Mux(kind),
            },
            ..Default::default()
        }
    }

    pub fn initiator_stream(kind: token::StreamKind, time: Timestamp) -> Self {
        P2pNetworkSelectState {
            time: Some(time),
            inner: P2pNetworkSelectStateInner::Initiator {
                proposing: token::Protocol::Stream(kind),
            },
            ..Default::default()
        }
    }

    pub fn default_timed(time: Timestamp) -> Self {
        P2pNetworkSelectState {
            time: Some(time),
            ..Default::default()
        }
    }

    pub fn is_timed_out(&self, now: Timestamp, timeouts: &P2pTimeouts) -> bool {
        if self.negotiated.is_some() {
            return false;
        }

        if let Some(time) = self.time {
            now.checked_sub(time)
                .and_then(|dur| timeouts.select.map(|to| dur >= to))
                .unwrap_or(false)
        } else {
            false
        }
    }

    pub fn is_incoming(&self) -> bool {
        matches!(&self.inner, P2pNetworkSelectStateInner::Responder)
    }

    /// Propagates incoming data to corresponding action
    pub(super) fn forward_incoming_data(
        &self,
        kind: SelectKind,
        addr: ConnectionAddr,
        data: Data,
        fin: bool,
    ) -> Vec<P2pNetworkSelectAction> {
        if self.negotiated.is_some() {
            vec![kind.forward_data(addr, data, fin)]
        } else {
            let mut tokens = vec![];
            let payload_data = &self.recv.buffer;
            let mut tokens_parsed = false;

            for token in &self.tokens {
                if !tokens_parsed {
                    tokens_parsed =
                        matches!(token, Token::Protocol(..) | Token::UnknownProtocol(..));
                }

                tokens.push(P2pNetworkSelectAction::IncomingToken { addr, kind });
            }

            if tokens_parsed && !payload_data.is_empty() {
                tokens.push(kind.forward_data(addr, Data::from(payload_data.clone()), fin));
            }

            tokens
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

use self::{p2p_network_select_state::P2pNetworkSelectStateInner, token::ParseTokenError};

use super::*;

impl P2pNetworkSelectState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkSelectAction>) {
        if let P2pNetworkSelectStateInner::Error(_) = &self.inner {
            return;
        }

        if self.negotiated.is_some() {
            self.reported = true;
        }

        let (action, meta) = action.split();
        match action {
            // hack for noise
            P2pNetworkSelectAction::Init { incoming, .. } => {
                self.time = Some(meta.time());

                match (&self.inner, incoming) {
                    (P2pNetworkSelectStateInner::Initiator { .. }, true) => {
                        self.inner = P2pNetworkSelectStateInner::Responder
                    }
                    (P2pNetworkSelectStateInner::Responder, false) => {
                        self.inner = P2pNetworkSelectStateInner::Initiator {
                            proposing: token::Protocol::Mux(token::MuxKind::YamuxNoNewLine1_0_0),
                        }
                    }
                    _ => {}
                }
            }
            P2pNetworkSelectAction::IncomingData { data, .. } => {
                if self.negotiated.is_none() {
                    self.recv.put(data);
                    loop {
                        let parse_result = self.recv.parse_token();

                        match parse_result {
                            Err(ParseTokenError) => {
                                self.inner =
                                    P2pNetworkSelectStateInner::Error("parse_token".to_owned());
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
            P2pNetworkSelectAction::IncomingPayload { .. } => self.recv.buffer.clear(),
            P2pNetworkSelectAction::IncomingToken { kind, .. } => {
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
                                token::Protocol::Mux(token::MuxKind::Yamux1_0_0) => {
                                    token::Token::Protocol(token::Protocol::Mux(
                                        token::MuxKind::Yamux1_0_0,
                                    ))
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

                                        openmina_core::error!(meta.time(); "unknown protocol: {str}, {kind:?}");
                                    }
                                } else {
                                    self.inner = P2pNetworkSelectStateInner::Error(format!(
                                        "responder with invalid protocol data {:?}",
                                        name
                                    ));

                                    openmina_core::error!(meta.time(); "invalid protocol: {name:?}, {kind:?}");
                                }
                            } else {
                                self.inner = P2pNetworkSelectStateInner::Error(
                                    "responder with empty protocol".to_string(),
                                );

                                openmina_core::error!(meta.time(); "empty protocol: {kind:?}");
                            }
                            self.to_send = Some(token::Token::Na);
                            self.negotiated = Some(None);
                        }
                    },
                }
            }
            P2pNetworkSelectAction::OutgoingTokens { .. } => {}
            P2pNetworkSelectAction::Timeout { .. } => {
                self.inner = P2pNetworkSelectStateInner::Error("timeout".to_string());
                openmina_core::warn!(meta.time(); "timeout");
            }
        }
    }
}

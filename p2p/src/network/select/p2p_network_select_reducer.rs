use self::p2p_network_select_state::P2pNetworkSelectStateInner;

use super::*;

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
            P2pNetworkSelectAction::Init { incoming, .. } => match (&self.inner, incoming) {
                (P2pNetworkSelectStateInner::Initiator { .. }, true) => {
                    self.inner = P2pNetworkSelectStateInner::Responder
                }
                (P2pNetworkSelectStateInner::Responder, false) => {
                    self.inner = P2pNetworkSelectStateInner::Initiator {
                        proposing: token::Protocol::Mux(token::MuxKind::YamuxNoNewLine1_0_0),
                    }
                }
                _ => {}
            },
            P2pNetworkSelectAction::IncomingData { data, .. } => {
                if self.negotiated.is_none() {
                    self.recv.put(data);
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
            P2pNetworkSelectAction::IncomingToken { .. } => {
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
                                token::Protocol::Stream(token::StreamKind::Broadcast(protocol)) => {
                                    token::Token::Protocol(token::Protocol::Stream(
                                        token::StreamKind::Broadcast(protocol),
                                    ))
                                }
                                token::Protocol::Stream(
                                    token::StreamKind::Rpc(_)
                                    | token::StreamKind::Discovery(_)
                                    | token::StreamKind::Identify(
                                        token::IdentifyAlgorithm::Identify1_0_0,
                                    ),
                                ) => token::Token::Protocol(protocol),
                                token::Protocol::Stream(
                                    token::StreamKind::Identify(
                                        token::IdentifyAlgorithm::IdentifyPush1_0_0,
                                    )
                                    | token::StreamKind::Ping(_)
                                    | token::StreamKind::Bitswap(_)
                                    | token::StreamKind::Status(_),
                                ) => token::Token::Na,
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
                                        openmina_core::error!(_meta.time(); "unknown protocol: {str}");
                                    }
                                }
                            }
                            self.to_send = Some(token::Token::Na);
                            self.negotiated = Some(None);
                        }
                    },
                }
            }
            P2pNetworkSelectAction::OutgoingTokens { .. } => {}
        }
    }
}

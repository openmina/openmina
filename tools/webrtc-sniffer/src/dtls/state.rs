use std::borrow::Cow;

use nom::{multi::many1, IResult, Parser};
use rand::{rngs::StdRng, SeedableRng};

use crate::dtls::handshake::{HandshakeInner, ServerHello};

use super::{
    handshake::{Extension, HandshakeMessage},
    header::{Chunk, ContentType},
};

pub struct State {
    rng_seed: [u8; 32],
    inner: Option<Inner>,
}

enum Inner {
    Initial,
    ClientHello {
        client_random: [u8; 32],
    },
    BothHello(HelloMsgs),
    ServerKey {
        hello: HelloMsgs,
        // https://www.iana.org/assignments/tls-parameters/tls-parameters.xhtml#tls-parameters-8
        curve_name: u16,
        server_pk: Vec<u8>,
    },
    BothKey {
        hello: HelloMsgs,
        keys: BothKey,
    },
}

#[allow(dead_code)]
struct HelloMsgs {
    client_random: [u8; 32],
    server_random: [u8; 32],
    session_id: Vec<u8>,
    cipher_suite: u16,
    extensions: Vec<Extension>,
}

#[allow(dead_code)]
struct BothKey {
    curve_name: u16,
    server_pk: Vec<u8>,
    client_pk: Vec<u8>,
}

impl State {
    pub fn new(rng_seed: [u8; 32]) -> Self {
        State {
            rng_seed,
            inner: Some(Inner::Initial),
        }
    }

    pub fn handle<'d>(&mut self, data: &'d [u8], _incoming: bool) -> IResult<&'d [u8], ()> {
        let (data, chunks) = many1(Chunk::parse).parse(data)?;
        for chunk in chunks {
            log::info!("{chunk}");
            #[allow(clippy::single_match)]
            match chunk.ty {
                ContentType::Handshake => self.handle_handshake(chunk.body),
                _ => {}
            }
        }

        Ok((data, ()))
    }

    fn handle_handshake(&mut self, msg_bytes: &[u8]) {
        let Some(state) = self.inner.take() else {
            log::warn!("ignore datagram, invalid state");
            return;
        };

        let mut msg_bytes = Cow::Borrowed(msg_bytes);
        if let Inner::BothKey { hello, keys } = &state {
            let bytes = msg_bytes.to_mut();
            // decrypt
            let _ = (hello, keys, bytes);
        }
        let msg = match HandshakeMessage::parse(&msg_bytes) {
            Ok((_, msg)) => msg,
            Err(err) => {
                log::error!("{err}");
                return;
            }
        };

        let HandshakeMessage {
            length,
            message_seq,
            fragment_offset,
            fragment_length,
            inner: msg,
        } = msg;
        let _ = message_seq;
        log::info!("HANDSHAKE: {msg}");

        if fragment_offset != 0 || length != fragment_length {
            log::error!("collecting fragments is not implemented");
            self.inner = None;
            return;
        }

        let state = match (state, msg) {
            (Inner::Initial, HandshakeInner::ClientHello(msg)) => {
                let client_random = msg.random;
                if msg.cookie.is_empty() {
                    self.inner = Some(Inner::Initial);
                    return;
                }

                use sha2::{
                    digest::{FixedOutput, Update},
                    Sha256,
                };
                let seed = Sha256::default()
                    .chain(self.rng_seed)
                    .chain(&msg.cookie)
                    .finalize_fixed()
                    .into();
                dbg!(format!("{seed:x?}"));
                let _rng = StdRng::from_seed(seed);

                let _ = (
                    msg.session_id,
                    msg.cookie,
                    msg.cipher_suites,
                    msg.compression_methods,
                    msg.extensions,
                );
                Inner::ClientHello { client_random }
            }
            (Inner::ClientHello { client_random }, HandshakeInner::ServerHello(msg)) => {
                let ServerHello {
                    random,
                    session_id,
                    cipher_suite,
                    compression_method,
                    extensions,
                } = msg;
                if compression_method != 0 {
                    log::error!("compression method {compression_method} is not implemented");
                    return;
                }
                Inner::BothHello(HelloMsgs {
                    client_random,
                    server_random: random,
                    session_id,
                    cipher_suite,
                    extensions,
                })
            }
            (Inner::BothHello(hello), HandshakeInner::ServerKeyExchange(msg)) => {
                // check signature
                let _ = msg.signature;
                Inner::ServerKey {
                    hello,
                    curve_name: msg.curve_name,
                    server_pk: msg.public_key,
                }
            }
            (
                Inner::ServerKey {
                    hello,
                    curve_name,
                    server_pk,
                },
                HandshakeInner::ClientKeyExchange(msg),
            ) => {
                let keys = BothKey {
                    curve_name,
                    server_pk,
                    client_pk: msg.public_key,
                };
                Inner::BothKey { hello, keys }
            }
            (state, _) => {
                log::warn!("ignore handshake msg");
                state
            }
        };
        self.inner = Some(state);
    }
}

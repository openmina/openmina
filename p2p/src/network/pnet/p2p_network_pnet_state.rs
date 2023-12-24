use redux::ActionMeta;
use salsa20::cipher::StreamCipher;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::P2pMioService;

use super::{super::*, *};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetState {
    shared_secret: [u8; 32],

    incoming: Half,
    outgoing: Half,
}

impl Drop for P2pNetworkPnetState {
    fn drop(&mut self) {
        self.shared_secret.zeroize();
    }
}

impl P2pNetworkPnetState {
    pub fn new(shared_secret: [u8; 32]) -> Self {
        P2pNetworkPnetState {
            shared_secret,

            incoming: Half::Buffering {
                buffer: [0; 24],
                offset: 0,
            },
            outgoing: Half::Buffering {
                buffer: [0; 24],
                offset: 0,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Half {
    Buffering {
        buffer: [u8; 24],
        offset: usize,
    },
    Done {
        cipher: XSalsa20Wrapper,
        to_send: Vec<u8>,
    },
}

impl Half {
    fn reduce(&mut self, shared_secret: &[u8; 32], data: &[u8]) {
        match self {
            Half::Buffering { buffer, offset } => {
                if *offset + data.len() < 24 {
                    buffer[*offset..(*offset + data.len())].clone_from_slice(data);
                    *offset += data.len();
                } else {
                    if *offset < 24 {
                        buffer[*offset..24].clone_from_slice(&data[..(24 - *offset)]);
                    }
                    let nonce = *buffer;
                    let remaining = data[(24 - *offset)..].to_vec().into_boxed_slice();
                    *self = Half::Done {
                        cipher: XSalsa20Wrapper::new(shared_secret, &nonce),
                        to_send: vec![],
                    };
                    self.reduce(shared_secret, &remaining);
                }
            }
            Half::Done { cipher, to_send } => {
                *to_send = data.to_vec();
                cipher.apply_keystream(to_send.as_mut());
            }
        }
    }
}

impl P2pNetworkPnetAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
        P2pNetworkSelectIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectInitAction: redux::EnablingCondition<S>,
    {
        let (state, service) = store.state_and_service();
        let connections = &state.network.connection.connections;
        let Some(state) = connections.get(&self.addr()) else {
            return;
        };
        match self {
            P2pNetworkPnetAction::IncomingData(a) => match &state.pnet.incoming {
                Half::Done { to_send, .. } if !to_send.is_empty() => {
                    let data = to_send.clone().into();
                    store.dispatch(P2pNetworkSelectIncomingDataAction {
                        addr: a.addr,
                        kind: SelectKind::Authentication,
                        data,
                    });
                }
                _ => {}
            },
            P2pNetworkPnetAction::OutgoingData(a) => match &state.pnet.outgoing {
                Half::Done { to_send, .. } if !to_send.is_empty() => {
                    service.send_mio_cmd(crate::MioCmd::Send(
                        a.addr,
                        to_send.clone().into_boxed_slice(),
                    ));
                }
                _ => {}
            },
            P2pNetworkPnetAction::SetupNonce(a) => {
                service.send_mio_cmd(crate::MioCmd::Send(
                    a.addr,
                    a.nonce.to_vec().into_boxed_slice(),
                ));
                store.dispatch(P2pNetworkSelectInitAction {
                    addr: a.addr,
                    kind: SelectKind::Authentication,
                    incoming: a.incoming,
                });
            }
        }
    }
}

impl P2pNetworkPnetState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkPnetAction>) {
        match action.action() {
            P2pNetworkPnetAction::IncomingData(a) => {
                self.incoming.reduce(&self.shared_secret, &a.data);
            }
            P2pNetworkPnetAction::OutgoingData(a) => {
                self.outgoing.reduce(&self.shared_secret, &a.data)
            }
            P2pNetworkPnetAction::SetupNonce(a) => {
                self.outgoing.reduce(&self.shared_secret, &a.nonce)
            }
        }
    }
}

use self::wrapper::XSalsa20Wrapper;
mod wrapper {
    use std::{
        fmt,
        ops::{Deref, DerefMut},
    };

    use salsa20::{cipher::generic_array::GenericArray, cipher::KeyIvInit, XSalsa20};

    #[derive(Clone)]
    pub struct XSalsa20Wrapper {
        inner: XSalsa20,
    }

    impl XSalsa20Wrapper {
        pub fn new(shared_secret: &[u8; 32], nonce: &[u8; 24]) -> Self {
            XSalsa20Wrapper {
                inner: XSalsa20::new(
                    GenericArray::from_slice(shared_secret),
                    GenericArray::from_slice(nonce),
                ),
            }
        }
    }

    impl Deref for XSalsa20Wrapper {
        type Target = XSalsa20;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl DerefMut for XSalsa20Wrapper {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }

    impl fmt::Debug for XSalsa20Wrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("XSalsa20").finish()
        }
    }

    impl serde::Serialize for XSalsa20Wrapper {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            unimplemented!()
        }
    }

    impl<'de> serde::Deserialize<'de> for XSalsa20Wrapper {
        fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            unimplemented!()
        }
    }
}

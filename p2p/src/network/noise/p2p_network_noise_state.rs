use std::collections::VecDeque;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{P2pCryptoService, P2pMioService};

use super::{super::*, *};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseState {
    pub buffer: Vec<u8>,
    pub incoming_chunks: VecDeque<Vec<u8>>,
    pub outgoing_chunks: VecDeque<Vec<u8>>,

    pub init: Option<P2pNetworkNoiseStateInitialized>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseStateInitialized {
    pub local_ephemeral: (Sk, Pk),
    pub local_static: (Sk, Pk),
    pub inner: P2pNetworkNoiseStateInner,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseStateInner {
    Initiator(P2pNetworkNoiseStateInitiator),
    Responder(P2pNetworkNoiseStateResponder),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseStateInitiator {
    Init,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseStateResponder {
    Init,
}

impl P2pNetworkNoiseAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingChunkAction: redux::EnablingCondition<S>,
    {
        let state = store.state();
        let Some(state) = state.network.connection.connections.get(&self.addr()) else {
            return;
        };
        let Some(P2pNetworkAuthState::Noise(state)) = &state.auth else {
            return;
        };

        let incoming = state.incoming_chunks.front().cloned().map(Into::into);
        let outgoing = state.outgoing_chunks.front().cloned().map(Into::into);

        if let Self::OutgoingChunk(a) = self {
            store.dispatch(P2pNetworkPnetOutgoingDataAction {
                addr: a.addr,
                data: a.data.clone(),
            });
        }
        if let Some(data) = incoming {
            store.dispatch(P2pNetworkNoiseIncomingChunkAction {
                addr: self.addr(),
                data,
            });
        }
        if let Some(data) = outgoing {
            store.dispatch(P2pNetworkNoiseOutgoingChunkAction {
                addr: self.addr(),
                data,
            });
        }
    }
}

impl P2pNetworkNoiseState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkNoiseAction>) {
        match action.action() {
            P2pNetworkNoiseAction::Init(a) => {
                let mut chunk = vec![0, 32];
                self.init = Some(P2pNetworkNoiseStateInitialized {
                    local_ephemeral: {
                        let sk = Sk::from(a.ephemeral_sk.clone());
                        let pk = Pk::from_sk(&sk);
                        chunk.extend_from_slice(&pk.0 .0);
                        (sk, pk)
                    },
                    local_static: {
                        let sk: Sk = Sk::from(a.static_sk.clone());
                        let pk = Pk::from_sk(&sk);
                        (sk, pk)
                    },
                    inner: if a.incoming {
                        P2pNetworkNoiseStateInner::Responder(P2pNetworkNoiseStateResponder::Init)
                    } else {
                        self.outgoing_chunks.push_back(chunk);
                        P2pNetworkNoiseStateInner::Initiator(P2pNetworkNoiseStateInitiator::Init)
                    },
                });
            }
            P2pNetworkNoiseAction::IncomingData(a) => {
                self.buffer.extend_from_slice(&a.data);
                let mut offset = 0;
                loop {
                    let buf = &self.buffer[offset..];
                    if buf.len() >= 2 {
                        let len = u16::from_be_bytes(buf[..2].try_into().expect("cannot fail"));
                        let full_len = 2 + len as usize;
                        if buf.len() >= full_len {
                            self.incoming_chunks.push_back(buf[..full_len].to_vec());
                            offset += full_len;
                            continue;
                        }
                    }
                    break;
                }
            }
            P2pNetworkNoiseAction::IncomingChunk(_) => {
                if let Some(chunk) = self.incoming_chunks.pop_front() {
                    //
                    let _ = chunk;
                }
            }
            P2pNetworkNoiseAction::OutgoingChunk(_) => {
                self.outgoing_chunks.pop_front();
            }
        }
    }
}

pub use self::wrapper::{Pk, Sk};
mod wrapper {
    use curve25519_dalek::{MontgomeryPoint, Scalar};
    use serde::{Deserialize, Serialize};
    use zeroize::Zeroize;

    use crate::DataSized;

    #[derive(Debug, Clone)]
    pub struct Pk(pub MontgomeryPoint);

    impl Pk {
        pub fn from_sk(sk: &Sk) -> Self {
            let t = curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
            Pk((t * &sk.0).to_montgomery())
        }
    }

    impl Serialize for Pk {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            hex::encode(self.0.as_bytes()).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Pk {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::Error;

            let str = <&'de str>::deserialize(deserializer)?;
            hex::decode(str)
                .map_err(Error::custom)
                .and_then(|b| b.try_into().map_err(|_| Error::custom("wrong length")))
                .map(MontgomeryPoint)
                .map(Self)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Sk(pub Scalar);

    impl From<DataSized<32>> for Sk {
        fn from(value: DataSized<32>) -> Self {
            Sk(Scalar::from_bytes_mod_order(value.0))
        }
    }

    impl Zeroize for Sk {
        fn zeroize(&mut self) {
            self.0.zeroize();
        }
    }

    impl Serialize for Sk {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            hex::encode(self.0.as_bytes()).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Sk {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::Error;

            let str = <&'de str>::deserialize(deserializer)?;
            hex::decode(str)
                .map_err(Error::custom)
                .and_then(|b| b.try_into().map_err(|_| Error::custom("wrong length")))
                .map(Scalar::from_bytes_mod_order)
                .map(Self)
        }
    }
}

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{P2pMioService, P2pNetworkPnetOutgoingDataAction};

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectState {
    pub kind: SelectKind,
    pub incoming: bool,

    pub recv: recv::State,
    pub tokens: Vec<Vec<u8>>,
    pub remaining: Vec<u8>,

    pub negotiated: Option<Vec<u8>>,
}

const NAME: &'static [u8] = b"\x13/multistream/1.0.0\n";
const SIM: &'static [u8] = b"\x1d/libp2p/simultaneous-connect\n";
// const NA: &'static [u8] = b"\x03na\n";

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
        match self {
            Self::Init(a) => match kind {
                SelectKind::Authentication => {
                    if state.incoming {
                        store.dispatch(P2pNetworkPnetOutgoingDataAction {
                            addr: a.addr,
                            data: NAME.to_vec().into_boxed_slice(),
                        });
                    } else {
                        let mut data = NAME.to_vec();
                        data.extend_from_slice(SIM);
                        data.extend_from_slice(kind.supported()[0]);
                        store.dispatch(P2pNetworkPnetOutgoingDataAction {
                            addr: a.addr,
                            data: data.into_boxed_slice(),
                        });
                    }
                }
                _ => unimplemented!(),
            },
            Self::IncomingData(a) => {
                let tokens = state
                    .tokens
                    .iter()
                    .map(|token| token.clone().into_boxed_slice())
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
                if a.token.as_ref() == NAME {
                    // do nothing, it is handshake token
                } else {
                    dbg!(std::str::from_utf8(&a.token)).unwrap_or_default();
                }
            }
        }
    }
}

impl P2pNetworkSelectState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkSelectAction>) {
        let (action, _meta) = action.split();
        match action {
            P2pNetworkSelectAction::Init(a) => {
                self.kind = action.select_kind();
                self.incoming = a.incoming;
            }
            P2pNetworkSelectAction::IncomingData(a) => {
                self.remaining.extend_from_slice(&a.data);
                if let Some(negotiated) = &self.negotiated {
                    // TODO: send to the negotiated handler
                    let _ = negotiated;
                } else {
                    loop {
                        let (token, remaining) = self.recv.parse_protocol(&self.remaining);
                        self.remaining = remaining.to_vec();
                        match token {
                            Some(v) => {
                                self.tokens.push(v.to_vec());
                                self.recv.consume();
                            }
                            None => break,
                        }
                    }
                }
            }
            P2pNetworkSelectAction::IncomingToken(_) => {
                self.tokens.remove(0);
            }
        }
    }
}

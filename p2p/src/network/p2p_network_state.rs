use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkState {
    pub connection: P2pNetworkConnectionState,
}

impl P2pNetworkState {
    pub fn new(chain_id: &str) -> Self {
        let pnet_key = {
            use blake2::{
                digest::{generic_array::GenericArray, Update, VariableOutput},
                Blake2bVar,
            };

            let mut key = GenericArray::default();
            Blake2bVar::new(32)
                .expect("valid constant")
                .chain(b"/coda/0.0.1/")
                .chain(chain_id.as_bytes())
                .finalize_variable(&mut key)
                .expect("good buffer size");
            key.into()
        };

        P2pNetworkState {
            connection: P2pNetworkConnectionState {
                interfaces: Default::default(),
                listeners: Default::default(),
                pnet_key,
                connections: Default::default(),
            },
        }
    }
}

impl P2pNetworkAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
        P2pNetworkPnetIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkPnetSetupNonceAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectInitAction: redux::EnablingCondition<S>,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingTokenAction: redux::EnablingCondition<S>,
        P2pNetworkConnectionSelectErrorAction: redux::EnablingCondition<S>,
        P2pNetworkConnectionSelectDoneAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseInitAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectOutgoingTokensAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingChunkAction: redux::EnablingCondition<S>,
    {
        match self {
            Self::Connection(v) => v.effects(meta, store),
            Self::Pnet(v) => v.effects(meta, store),
            Self::Select(v) => v.effects(meta, store),
            Self::Noise(v) => v.effects(meta, store),
        }
    }
}

impl P2pNetworkState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkAction>) {
        let (action, meta) = action.split();
        match action {
            P2pNetworkAction::Connection(a) => self.connection.reducer(meta.with_action(&a)),
            P2pNetworkAction::Pnet(a) => {
                self.connection
                    .connections
                    .get_mut(&a.addr())
                    .map(|cn| cn.pnet.reducer(meta.with_action(&a)));
            }
            P2pNetworkAction::Select(a) => {
                self.connection
                    .connections
                    .get_mut(&a.addr())
                    .map(|cn| match a.id() {
                        SelectKind::Authentication => cn.select_auth.reducer(meta.with_action(&a)),
                        SelectKind::Multiplexing(_) => cn.select_mux.reducer(meta.with_action(&a)),
                        SelectKind::Stream(_, stream_id) => {
                            cn.streams
                                .get_mut(&stream_id)
                                .map(|stream| stream.select.reducer(meta.with_action(&a)));
                        }
                    });
            }
            P2pNetworkAction::Noise(a) => {
                self.connection
                    .connections
                    .get_mut(&a.addr())
                    .map(|cn| match &mut cn.auth {
                        Some(P2pNetworkAuthState::Noise(state)) => {
                            state.reducer(meta.with_action(&a))
                        }
                        _ => {}
                    });
            }
        }
    }
}

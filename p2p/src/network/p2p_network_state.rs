use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkState {
    pub scheduler: P2pNetworkSchedulerState,
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
            scheduler: P2pNetworkSchedulerState {
                interfaces: Default::default(),
                listeners: Default::default(),
                pnet_key,
                connections: Default::default(),
                broadcast_state: Default::default(),
                discovery_state: Default::default(),
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
        P2pNetworkSchedulerSelectErrorAction: redux::EnablingCondition<S>,
        P2pNetworkSchedulerSelectDoneAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseInitAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectOutgoingTokensAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseDecryptedDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseHandshakeDoneAction: redux::EnablingCondition<S>,
        P2pNetworkSchedulerYamuxDidInitAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxIncomingFrameAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxPingStreamAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOpenStreamAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOutgoingFrameAction: redux::EnablingCondition<S>,
        P2pNetworkRpcInitAction: redux::EnablingCondition<S>,
        P2pNetworkRpcIncomingDataAction: redux::EnablingCondition<S>,
    {
        match self {
            Self::Scheduler(v) => v.effects(meta, store),
            Self::Pnet(v) => v.effects(meta, store),
            Self::Select(v) => v.effects(meta, store),
            Self::Noise(v) => v.effects(meta, store),
            Self::Yamux(v) => v.effects(meta, store),
            Self::Rpc(v) => v.effects(meta, store),
        }
    }
}

impl P2pNetworkState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkAction>) {
        let (action, meta) = action.split();
        match action {
            P2pNetworkAction::Scheduler(a) => self.scheduler.reducer(meta.with_action(&a)),
            P2pNetworkAction::Pnet(a) => {
                self.scheduler
                    .connections
                    .get_mut(&a.addr())
                    .map(|cn| cn.pnet.reducer(meta.with_action(&a)));
            }
            P2pNetworkAction::Select(a) => {
                self.scheduler
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
                self.scheduler
                    .connections
                    .get_mut(&a.addr())
                    .map(|cn| match &mut cn.auth {
                        Some(P2pNetworkAuthState::Noise(state)) => {
                            state.reducer(meta.with_action(&a))
                        }
                        _ => {}
                    });
            }
            P2pNetworkAction::Yamux(a) => {
                self.scheduler
                    .connections
                    .get_mut(&a.addr())
                    .map(|cn| match &mut cn.mux {
                        Some(P2pNetworkConnectionMuxState::Yamux(state)) => {
                            state.reducer(&mut cn.streams, meta.with_action(&a))
                        }
                        _ => {}
                    });
            }
            P2pNetworkAction::Rpc(a) => {
                self.scheduler
                    .connections
                    .get_mut(&a.addr())
                    .and_then(|cn| cn.streams.get_mut(&a.stream_id()))
                    .and_then(|stream| stream.handler.as_mut())
                    .map(|handler| match handler {
                        P2pNetworkStreamHandlerState::Rpc(state) => {
                            state.reducer(meta.with_action(&a))
                        }
                        _ => {}
                    });
            }
        }
    }
}

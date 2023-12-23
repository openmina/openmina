use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{P2pMioService, P2pNetworkPnetOutgoingDataAction};

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectState {
    pub kind: SelectKind,
    pub recv: recv::State,
    pub incoming: bool,

    pub handshake_sent: bool,
    pub handshake_received: bool,
}

const NAME: &'static [u8] = b"\x13/multistream/1.0.0\n";
const SIM: &'static [u8] = b"\x1d/libp2p/simultaneous-connect\n";
const NA: &'static [u8] = b"\x03na\n";

impl P2pNetworkSelectAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
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
                    store.dispatch(P2pNetworkPnetOutgoingDataAction {
                        addr: a.addr,
                        data: NAME.to_vec().into_boxed_slice(),
                    });
                    store.dispatch(P2pNetworkPnetOutgoingDataAction {
                        addr: a.addr,
                        data: SIM.to_vec().into_boxed_slice(),
                    });
                    store.dispatch(P2pNetworkPnetOutgoingDataAction {
                        addr: a.addr,
                        data: kind.supported()[0].to_vec().into_boxed_slice(),
                    });
                }
                _ => unimplemented!(),
            },
            Self::IncomingData(a) => {
                dbg!(std::str::from_utf8(&a.data[1..])).unwrap_or_default();
            }
        }

        let _ = store;
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
                let _ = a;
            }
        }
    }
}

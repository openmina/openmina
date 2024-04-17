mod pb {
    include!(concat!(env!("OUT_DIR"), "/gossipsub.pb.rs"));
}

use std::{collections::BTreeMap, net::SocketAddr};

use openmina_core::ActionEvent;

use serde::{Deserialize, Serialize};

use crate::{token::BroadcastAlgorithm, Data, P2pPeerState, P2pState, PeerId};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubState {
    //
}

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkPubsubAction {
    NewStream {
        incoming: bool,
        addr: SocketAddr,
        protocol: BroadcastAlgorithm,
    },
    IncomingData {
        addr: SocketAddr,
        data: Data,
    },
}

impl From<P2pNetworkPubsubAction> for crate::P2pAction {
    fn from(value: P2pNetworkPubsubAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPubsubAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl P2pNetworkPubsubState {
    pub fn reducer(
        &mut self,
        peers: &mut BTreeMap<PeerId, P2pPeerState>,
        action: redux::ActionWithMeta<&P2pNetworkPubsubAction>,
    ) {
        let _ = (peers, action);
    }
}

impl P2pNetworkPubsubAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let _ = (meta, store);
        match self {
            Self::NewStream {
                incoming,
                addr,
                protocol,
            } => {
                dbg!((addr, protocol.name_str(), incoming));
            }
            Self::IncomingData { addr, data } => {
                dbg!((addr, data));
            }
        }
    }
}

use std::net::{IpAddr, SocketAddr};

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{MioCmd, P2pMioService, P2pState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkConnectionAction {
    InterfaceDetected(IpAddr),
    InterfaceExpired(IpAddr),
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        match self {
            Self::InterfaceDetected(_) => true,
            Self::InterfaceExpired(_) => true,
        }
    }
}

impl P2pNetworkConnectionAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
    {
        match self {
            Self::InterfaceDetected(ip) => {
                let port = store.state().config.libp2p_port.unwrap_or_default();
                store
                    .service()
                    .send_mio_cmd(MioCmd::ListenOn(SocketAddr::new(*ip, port)));
            }
            Self::InterfaceExpired(_) => {}
        }
    }
}

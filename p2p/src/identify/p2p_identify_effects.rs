use crate::{
    connection::P2pConnectionService, P2pNetworkConnectionMuxState, P2pNetworkKademliaAction,
    P2pNetworkYamuxAction, P2pStore,
};
use openmina_core::error;
use redux::ActionMeta;

use super::P2pIdentifyAction;

impl P2pIdentifyAction {
    pub fn effects<S, Store>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pIdentifyAction::NewRequest { addr, .. } => {
                let scheduler = &store.state().network.scheduler;
                let stream_id = scheduler
                    .connections
                    .get(&addr)
                    .ok_or_else(|| format!("connection with {addr} not found"))
                    .and_then(|conn| {
                        conn.mux
                            .as_ref()
                            .map(|mux| (mux, conn.incoming))
                            .ok_or_else(|| format!("multiplexing is not ready for {addr}"))
                    })
                    .and_then(|(P2pNetworkConnectionMuxState::Yamux(yamux), incoming)| {
                        yamux
                            .next_stream_id(crate::YamuxStreamKind::Identify, incoming)
                            .ok_or_else(|| format!("cannot get next stream for {addr}"))
                    });

                match stream_id {
                    Ok(stream_id) => {
                        store.dispatch(P2pNetworkYamuxAction::OpenStream {
                            addr,
                            stream_id,
                            stream_kind: crate::token::StreamKind::Identify(
                                crate::token::IdentifyAlgorithm::Identify1_0_0,
                            ),
                        });
                    }
                    Err(e) => {
                        error!(meta.time(); "error dispatching Identify action: {e}");
                    }
                }
            }
            P2pIdentifyAction::UpdatePeerInformation { peer_id, info } => {
                store.dispatch(P2pNetworkKademliaAction::UpdateRoutingTable {
                    peer_id,
                    addrs: info.listen_addrs,
                });
            }
        }
    }
}

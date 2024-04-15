use crate::{
    connection::P2pConnectionService, P2pNetworkConnectionMuxState, P2pNetworkYamuxAction, P2pStore,
};
use openmina_core::error;
use redux::ActionMeta;

use super::P2pFloodsubAction;

impl P2pFloodsubAction {
    pub fn effects<S, Store>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pFloodsubAction::NewOutboundStream { addr, .. } => {
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
                            .next_stream_id(!incoming)
                            .ok_or_else(|| format!("cannot get next stream for {addr}"))
                    });

                match stream_id {
                    Ok(stream_id) => {
                        store.dispatch(P2pNetworkYamuxAction::OpenStream {
                            addr,
                            stream_id,
                            stream_kind: crate::token::StreamKind::Broadcast(
                                //crate::token::BroadcastAlgorithm::Floodsub1_0_0,
                                crate::token::BroadcastAlgorithm::Meshsub1_0_0,
                            ),
                        });
                    }
                    Err(e) => {
                        error!(meta.time(); "error dispatching Floodsub action: {e}");
                    }
                }
            }
        }
    }
}

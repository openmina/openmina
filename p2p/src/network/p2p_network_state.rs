use multiaddr::Multiaddr;
use openmina_core::ChainId;
use serde::{Deserialize, Serialize};

use crate::{identity::PublicKey, PeerId};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkState {
    pub scheduler: P2pNetworkSchedulerState,
}

impl P2pNetworkState {
    pub fn new(
        identity: PublicKey,
        addrs: Vec<Multiaddr>,
        known_peers: Vec<(PeerId, Multiaddr)>,
        chain_id: &ChainId,
        discovery: bool,
    ) -> Self {
        let peer_id = identity.peer_id();
        let pnet_key = chain_id.preshared_key();
        let discovery_state = discovery.then(|| {
            let mut routing_table =
                P2pNetworkKadRoutingTable::new(P2pNetworkKadEntry::new(peer_id, addrs));
            routing_table.extend(
                known_peers
                    .into_iter()
                    .map(|(peer_id, maddr)| P2pNetworkKadEntry::new(peer_id, vec![maddr])),
            );
            P2pNetworkKadState {
                routing_table,
                ..Default::default()
            }
        });

        P2pNetworkState {
            scheduler: P2pNetworkSchedulerState {
                interfaces: Default::default(),
                listeners: Default::default(),
                local_pk: identity,
                pnet_key,
                connections: Default::default(),
                broadcast_state: Default::default(),
                identify_state: Default::default(),
                discovery_state,
                rpc_incoming_streams: Default::default(),
                rpc_outgoing_streams: Default::default(),
            },
        }
    }

    pub fn find_rpc_state(&self, a: &P2pNetworkRpcAction) -> Option<&P2pNetworkRpcState> {
        match a.stream_id() {
            RpcStreamId::Exact(stream_id) => self
                .scheduler
                .rpc_incoming_streams
                .get(a.peer_id())
                .and_then(|cn| cn.get(&stream_id))
                .or_else(|| {
                    self.scheduler
                        .rpc_outgoing_streams
                        .get(a.peer_id())
                        .and_then(|cn| cn.get(&stream_id))
                }),
            RpcStreamId::WithQuery(id) => self
                .scheduler
                .rpc_incoming_streams
                .get(a.peer_id())
                .and_then(|streams| {
                    streams.iter().find_map(|(_, state)| {
                        if state
                            .pending
                            .as_ref()
                            .map_or(false, |query_header| query_header.id == id)
                        {
                            Some(state)
                        } else {
                            None
                        }
                    })
                }),
            RpcStreamId::AnyIncoming => self
                .scheduler
                .rpc_incoming_streams
                .get(a.peer_id())
                .and_then(|stream| stream.first_key_value())
                .map(|(_k, v)| v),
            RpcStreamId::AnyOutgoing => {
                if let Some(streams) = self.scheduler.rpc_outgoing_streams.get(a.peer_id()) {
                    if let Some((k, _)) = streams.first_key_value() {
                        return Some(streams.get(k).expect("checked above"));
                    }
                }

                None
            }
        }
    }
}

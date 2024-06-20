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
}

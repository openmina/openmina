use multiaddr::Multiaddr;
use openmina_core::error;

use crate::{identity::PublicKey, PeerId};

use super::*;

impl P2pNetworkState {
    pub fn new(
        identity: PublicKey,
        addrs: Vec<Multiaddr>,
        known_peers: Vec<(PeerId, Multiaddr)>,
        chain_id: &str,
        discovery: bool,
    ) -> Self {
        let peer_id = identity.peer_id();
        let pnet_key = {
            use blake2::{
                digest::{generic_array::GenericArray, Update, VariableOutput},
                Blake2bVar,
            };

            let mut key = GenericArray::default();
            Blake2bVar::new(32)
                .expect("valid constant")
                .chain(b"/coda/0.0.1/")
                .chain(chain_id)
                .finalize_variable(&mut key)
                .expect("good buffer size");
            key.into()
        };

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

impl P2pNetworkState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkAction>) {
        let (action, meta) = action.split();
        match action {
            P2pNetworkAction::Scheduler(a) => self.scheduler.reducer(meta.with_action(a)),
            P2pNetworkAction::Pnet(a) => {
                if let Some(cn) = self.scheduler.connections.get_mut(a.addr()) {
                    cn.pnet.reducer(meta.with_action(a))
                }
            }
            P2pNetworkAction::Select(a) => {
                if let Some(cn) = self.scheduler.connections.get_mut(a.addr()) {
                    match a.id() {
                        SelectKind::Authentication => cn.select_auth.reducer(meta.with_action(a)),
                        SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                            cn.select_mux.reducer(meta.with_action(a))
                        }
                        SelectKind::Stream(_, stream_id) => {
                            if let Some(stream) = cn.streams.get_mut(stream_id) {
                                stream.select.reducer(meta.with_action(a))
                            }
                        }
                    }
                }
            }
            P2pNetworkAction::Noise(a) => {
                if let Some(cn) = self.scheduler.connections.get_mut(a.addr()) {
                    if let Some(P2pNetworkAuthState::Noise(state)) = &mut cn.auth {
                        state.reducer(meta.with_action(a))
                    }
                }
            }
            P2pNetworkAction::Yamux(a) => {
                if let Some(cn) = self.scheduler.connections.get_mut(a.addr()) {
                    if let Some(P2pNetworkConnectionMuxState::Yamux(state)) = &mut cn.mux {
                        state.reducer(&mut cn.streams, meta.with_action(a))
                    }
                }
            }
            P2pNetworkAction::Identify(a) => {
                let time = meta.time();
                // println!("======= identify reducer for {state:?}");
                if let Err(err) = self.scheduler.identify_state.reducer(meta.with_action(&a)) {
                    error!(time; "{err}");
                }
                // println!("======= identify reducer result {state:?}");
            }
            P2pNetworkAction::Kad(a) => {
                let Some(state) = &mut self.scheduler.discovery_state else {
                    error!(meta.time(); "kademlia is not configured");
                    return;
                };
                let time = meta.time();
                // println!("======= kad reducer for {state:?}");
                if let Err(err) = state.reducer(meta.with_action(a)) {
                    error!(time; "{err}");
                }
                // println!("======= kad reducer result {state:?}");
            }
            P2pNetworkAction::Pubsub(a) => self
                .scheduler
                .broadcast_state
                .reducer(peers, meta.with_action(a)),
            P2pNetworkAction::Rpc(a) => {
                if let Some(state) = self.find_rpc_state_mut(a) {
                    state.reducer(meta.with_action(a))
                }
            }
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

    pub fn find_rpc_state_mut(
        &mut self,
        a: &P2pNetworkRpcAction,
    ) -> Option<&mut P2pNetworkRpcState> {
        match a.stream_id() {
            RpcStreamId::Exact(stream_id) => self
                .scheduler
                .rpc_incoming_streams
                .get_mut(a.peer_id())
                .and_then(|cn| cn.get_mut(&stream_id))
                .or_else(|| {
                    self.scheduler
                        .rpc_outgoing_streams
                        .get_mut(a.peer_id())
                        .and_then(|cn| cn.get_mut(&stream_id))
                }),
            RpcStreamId::AnyIncoming => {
                if let Some(streams) = self.scheduler.rpc_incoming_streams.get_mut(a.peer_id()) {
                    if let Some((k, _)) = streams.first_key_value() {
                        let k = *k;
                        return Some(streams.get_mut(&k).expect("checked above"));
                    }
                }

                None
            }
            RpcStreamId::AnyOutgoing => {
                if let Some(streams) = self.scheduler.rpc_outgoing_streams.get_mut(a.peer_id()) {
                    if let Some((k, _)) = streams.first_key_value() {
                        let k = *k;
                        return Some(streams.get_mut(&k).expect("checked above"));
                    }
                }

                None
            }
        }
    }
}

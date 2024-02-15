use std::collections::BTreeMap;

use multiaddr::Multiaddr;
use openmina_core::error;
use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::channels::rpc::P2pChannelsRpcAction;
use crate::connection::outgoing::P2pConnectionOutgoingAction;
use crate::{P2pPeerState, P2pPeerStatus, PeerId};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkState {
    pub scheduler: P2pNetworkSchedulerState,
}

impl P2pNetworkState {
    pub fn new(
        peer_id: PeerId,
        addrs: Vec<Multiaddr>,
        known_peers: Vec<(PeerId, Multiaddr)>,
        chain_id: &[u8],
        discovery: bool,
    ) -> Self {
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
                pnet_key,
                connections: Default::default(),
                broadcast_state: Default::default(),
                discovery_state,
                rpc_incoming_streams: Default::default(),
                rpc_outgoing_streams: Default::default(),
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
        P2pNetworkRpcOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkRpcIncomingMessageAction: redux::EnablingCondition<S>,
        P2pNetworkRpcOutgoingQueryAction: redux::EnablingCondition<S>,
        P2pChannelsRpcAction: redux::EnablingCondition<S>,
        P2pNetworkKademliaAction: redux::EnablingCondition<S>,
        P2pConnectionOutgoingAction: redux::EnablingCondition<S>,
        super::kad::bootstrap::P2pNetworkKadBootstrapAction: redux::EnablingCondition<S>,
        super::kad::request::P2pNetworkKadRequestAction: redux::EnablingCondition<S>,
        super::kad::P2pNetworkKademliaStreamAction: redux::EnablingCondition<S>,
    {
        match self {
            Self::Scheduler(v) => v.effects(meta, store),
            Self::Pnet(v) => v.effects(meta, store),
            Self::Select(v) => v.effects(meta, store),
            Self::Noise(v) => v.effects(meta, store),
            Self::Yamux(v) => v.effects(meta, store),
            Self::Kad(v) => match v.effects(meta, store) {
                Ok(_) => {}
                Err(e) => error!(meta.time(); "error dispatching Kademlia stream action: {e}"),
            },
            Self::Rpc(v) => v.effects(meta, store),
        }
    }
}

impl P2pNetworkState {
    pub fn reducer(
        &mut self,
        peers: &mut BTreeMap<PeerId, P2pPeerState>,
        action: redux::ActionWithMeta<&P2pNetworkAction>,
    ) {
        let (action, meta) = action.split();
        match action {
            P2pNetworkAction::Scheduler(a) => self.scheduler.reducer(peers, meta.with_action(&a)),
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
                        SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                            cn.select_mux.reducer(meta.with_action(&a))
                        }
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
            P2pNetworkAction::Kad(a) => {
                let Some(state) = &mut self.scheduler.discovery_state else {
                    error!(meta.time(); "kademlia is not configured");
                    return;
                };
                let time = meta.time();
                // println!("======= kad reducer for {state:?}");
                if let Err(err) = state.reducer(meta.with_action(&a)) {
                    error!(time; "{err}");
                }
                // println!("======= kad reducer result {state:?}");
            }
            P2pNetworkAction::Rpc(a) => {
                if let Some(state) = self.find_rpc_state_mut(a) {
                    if let Some(peer_state) = peers.get_mut(&a.peer_id()) {
                        if let P2pPeerStatus::Ready(status) = &mut peer_state.status {
                            state.reducer(&mut status.channels.rpc, meta.with_action(&a))
                        }
                    }
                }
            }
        }
    }

    pub fn find_rpc_state(&self, a: &P2pNetworkRpcAction) -> Option<&P2pNetworkRpcState> {
        match a.stream_id() {
            RpcStreamId::Exact(stream_id) => self
                .scheduler
                .rpc_incoming_streams
                .get(&a.peer_id())
                .and_then(|cn| cn.get(&stream_id))
                .or_else(|| {
                    self.scheduler
                        .rpc_outgoing_streams
                        .get(&a.peer_id())
                        .and_then(|cn| cn.get(&stream_id))
                }),
            RpcStreamId::AnyIncoming => self
                .scheduler
                .rpc_incoming_streams
                .get(&a.peer_id())
                .and_then(|stream| stream.first_key_value())
                .map(|(_k, v)| v),
            RpcStreamId::AnyOutgoing => {
                if let Some(streams) = self.scheduler.rpc_outgoing_streams.get(&a.peer_id()) {
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
                .get_mut(&a.peer_id())
                .and_then(|cn| cn.get_mut(&stream_id))
                .or_else(|| {
                    self.scheduler
                        .rpc_outgoing_streams
                        .get_mut(&a.peer_id())
                        .and_then(|cn| cn.get_mut(&stream_id))
                }),
            RpcStreamId::AnyIncoming => {
                if let Some(streams) = self.scheduler.rpc_incoming_streams.get_mut(&a.peer_id()) {
                    if let Some((k, _)) = streams.first_key_value() {
                        let k = *k;
                        return Some(streams.get_mut(&k).expect("checked above"));
                    }
                }

                None
            }
            RpcStreamId::AnyOutgoing => {
                if let Some(streams) = self.scheduler.rpc_outgoing_streams.get_mut(&a.peer_id()) {
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

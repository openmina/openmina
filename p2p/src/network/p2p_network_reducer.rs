use openmina_core::log::error;

use crate::P2pLimits;

use super::*;

impl P2pNetworkState {
    pub fn reducer(
        &mut self,
        action: redux::ActionWithMeta<&P2pNetworkAction>,
        limits: &P2pLimits,
    ) {
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
                    match a.select_kind() {
                        SelectKind::Authentication => cn.select_auth.reducer(meta.with_action(a)),
                        SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                            cn.select_mux.reducer(meta.with_action(a))
                        }
                        SelectKind::Stream(_, stream_id) => {
                            if let Some(stream) = cn.streams.get_mut(&stream_id) {
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
                if let Err(err) = self
                    .scheduler
                    .identify_state
                    .reducer(meta.with_action(a), limits)
                {
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
                if let Err(err) = state.reducer(meta.with_action(a), limits) {
                    error!(time; "{err}");
                }
            }
            P2pNetworkAction::Pubsub(a) => {
                self.scheduler.broadcast_state.reducer(meta.with_action(a))
            }
            P2pNetworkAction::Rpc(a) => {
                if let Some(state) = self.find_rpc_state_mut(a) {
                    state.reducer(meta.with_action(a), limits)
                }
            }
        }
    }

    fn find_rpc_state_mut(&mut self, a: &P2pNetworkRpcAction) -> Option<&mut P2pNetworkRpcState> {
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
            RpcStreamId::WithQuery(id) => self
                .scheduler
                .rpc_incoming_streams
                .get_mut(a.peer_id())
                .and_then(|streams| {
                    streams.iter_mut().find_map(|(_, state)| {
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

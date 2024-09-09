use crate::P2pLimits;
use identify::P2pNetworkIdentifyState;
use openmina_core::Substate;

use super::*;

impl P2pNetworkState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, Self>,
        action: redux::ActionWithMeta<&P2pNetworkAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let state = state_context.get_substate_mut()?;

        let (action, meta) = action.split();
        match action {
            P2pNetworkAction::Scheduler(a) => {
                state.scheduler.reducer(meta.with_action(a));
                Ok(())
            }
            P2pNetworkAction::Pnet(a) => {
                if let Some(cn) = state.scheduler.connections.get_mut(a.addr()) {
                    cn.pnet.reducer(meta.with_action(a))
                }
                Ok(())
            }
            P2pNetworkAction::Select(a) => {
                if let Some(cn) = state.scheduler.connections.get_mut(a.addr()) {
                    match a.select_kind() {
                        SelectKind::Authentication => cn.select_auth.reducer(meta.with_action(a)),
                        SelectKind::Multiplexing(_) | SelectKind::MultiplexingNoPeerId => {
                            cn.select_mux.reducer(meta.with_action(a));
                        }
                        SelectKind::Stream(_, stream_id) => {
                            if let Some(stream) = cn.streams.get_mut(&stream_id) {
                                stream.select.reducer(meta.with_action(a))
                            }
                        }
                    };
                }

                Ok(())
            }
            P2pNetworkAction::Noise(a) => P2pNetworkNoiseState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(a),
            ),
            P2pNetworkAction::Yamux(a) => {
                // if let Some(cn) = state.scheduler.connections.get_mut(a.addr()) {
                //     if let Some(P2pNetworkConnectionMuxState::Yamux(state)) = &mut cn.mux {
                //         state.reducer(&mut cn.streams, meta.with_action(a))?;
                //     }
                // }
                // Ok(())
                P2pNetworkYamuxState::reducer(
                    Substate::from_compatible_substate(state_context),
                    meta.with_action(a),
                )
            }
            P2pNetworkAction::Identify(a) => P2pNetworkIdentifyState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(a),
                limits,
            ),
            P2pNetworkAction::Kad(a) => P2pNetworkKadState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(a),
                limits,
            ),
            P2pNetworkAction::Pubsub(a) => {
                state.scheduler.broadcast_state.reducer(meta.with_action(a));
                Ok(())
            }
            P2pNetworkAction::Rpc(a) => {
                if let Some(state) = state.find_rpc_state_mut(a) {
                    state.reducer(meta.with_action(a), limits);
                }

                Ok(())
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

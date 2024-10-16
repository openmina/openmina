use crate::{
    channels::{
        rpc::P2pChannelsRpcAction, signaling::discovery::P2pChannelsSignalingDiscoveryAction,
        streaming_rpc::P2pChannelsStreamingRpcAction, P2pChannelsState,
    },
    connection::{
        incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction,
        P2pConnectionState,
    },
    disconnection::P2pDisconnectedState,
    P2pAction, P2pNetworkKadKey, P2pNetworkKademliaAction, P2pNetworkPnetAction,
    P2pNetworkRpcAction, P2pNetworkSelectAction, P2pNetworkState, P2pPeerState, P2pState, PeerId,
};
use openmina_core::{bug_condition, Substate};
use redux::{ActionMeta, ActionWithMeta, Dispatcher, Timestamp};

impl P2pState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, Self>,
        action: ActionWithMeta<P2pAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let Ok(state) = state_context.get_substate_mut() else {
            bug_condition!("no P2pState");
            return Ok(());
        };
        let (action, meta) = action.split();

        match action {
            P2pAction::Initialization(_) => {
                // noop
                Ok(())
            }
            P2pAction::Connection(action) => {
                P2pConnectionState::reducer(state_context, meta.with_action(action))
            }
            P2pAction::Disconnection(action) => {
                P2pDisconnectedState::reducer(state_context, meta.with_action(action))
            }
            P2pAction::Peer(action) => P2pPeerState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(action),
            ),
            P2pAction::Channels(action) => {
                P2pChannelsState::reducer(state_context, meta.with_action(action))
            }
            P2pAction::Identify(_action) => {
                #[cfg(feature = "p2p-libp2p")]
                Self::identify_reducer(state_context, meta.with_action(_action))?;
                Ok(())
            }
            P2pAction::Network(_action) => {
                #[cfg(feature = "p2p-libp2p")]
                {
                    let limits = state.config.limits;
                    P2pNetworkState::reducer(
                        Substate::from_compatible_substate(state_context),
                        meta.with_action(_action),
                        &limits,
                    )?;
                }
                Ok(())
            }
        }
    }

    pub fn p2p_timeout_dispatch<State, Action>(
        state_context: Substate<Action, State, Self>,
        meta: &ActionMeta,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (dispatcher, state) = state_context.into_dispatcher_and_state();
        let state: &P2pState = state.substate()?;
        let time = meta.time();

        state.p2p_connection_timeouts_dispatch(dispatcher, time)?;
        dispatcher.push(P2pConnectionOutgoingAction::RandomInit);

        state.p2p_try_reconnect_disconnected_peers(dispatcher, time)?;
        state.p2p_discovery(dispatcher, time)?;

        #[cfg(feature = "p2p-libp2p")]
        {
            state.p2p_pnet_timeouts(dispatcher, time)?;
            state.p2p_select_timeouts(dispatcher, time)?;
            state.p2p_rpc_heartbeats(dispatcher, time)?;
        }

        state.rpc_timeouts(dispatcher, time)?;
        Ok(())
    }

    fn p2p_connection_timeouts_dispatch<State, Action>(
        &self,
        dispatcher: &mut Dispatcher<Action, State>,
        time: Timestamp,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let timeouts = &self.config.timeouts;

        self.peers
            .iter()
            .filter_map(|(peer_id, peer)| {
                let state = peer.status.as_connecting()?;
                state
                    .is_timed_out(time, timeouts)
                    .then(|| (*peer_id, state.as_outgoing().is_some()))
            })
            .for_each(|(peer_id, is_outgoing)| match is_outgoing {
                true => dispatcher.push(P2pConnectionOutgoingAction::Timeout { peer_id }),
                false => dispatcher.push(P2pConnectionIncomingAction::Timeout { peer_id }),
            });

        Ok(())
    }

    fn p2p_try_reconnect_disconnected_peers<State, Action>(
        &self,
        dispatcher: &mut Dispatcher<Action, State>,
        time: Timestamp,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        if self.already_has_min_peers() {
            return Ok(());
        }

        let timeouts = &self.config.timeouts;

        self.peers
            .iter()
            .filter_map(|(_, peer)| {
                if peer.can_reconnect(time, timeouts) {
                    peer.dial_opts.clone()
                } else {
                    None
                }
            })
            .map(|opts| P2pConnectionOutgoingAction::Reconnect { opts, rpc_id: None })
            .for_each(|action| dispatcher.push(action));
        Ok(())
    }

    fn rpc_timeouts<State, Action>(
        &self,
        dispatcher: &mut Dispatcher<Action, State>,
        time: Timestamp,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        self.peer_rpc_timeouts(time)
            .into_iter()
            .for_each(|(peer_id, id, is_streaming)| {
                if is_streaming {
                    dispatcher.push(P2pChannelsStreamingRpcAction::Timeout { peer_id, id });
                } else {
                    dispatcher.push(P2pChannelsRpcAction::Timeout { peer_id, id });
                }
            });

        Ok(())
    }

    fn p2p_discovery<State, Action>(
        &self,
        dispatcher: &mut Dispatcher<Action, State>,
        time: Timestamp,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let config = &self.config;
        let timeouts = &config.timeouts;

        if !config.peer_discovery {
            return Ok(());
        }

        for (&peer_id, _) in self
            .ready_peers_iter()
            .filter(|(_, peer)| peer.channels.signaling.discovery.is_ready())
        {
            dispatcher.push(P2pChannelsSignalingDiscoveryAction::RequestSend { peer_id });
            dispatcher.push(P2pChannelsSignalingDiscoveryAction::DiscoveryRequestSend { peer_id });
        }

        if let Some(_d) = config.timeouts.initial_peers {
            // ask initial peers
            // TODO: use RPC to ask initial peers
        }

        #[cfg(feature = "p2p-libp2p")]
        {
            if let Some(discovery_state) = self.network.scheduler.discovery_state() {
                let my_id = self.my_id();

                match P2pNetworkKadKey::try_from(&my_id) {
                    Ok(key) => {
                        if discovery_state.status.can_bootstrap(time, timeouts)
                            && discovery_state
                                .routing_table
                                .closest_peers(&key)
                                .any(|_| true)
                        {
                            dispatcher
                                .push(P2pNetworkKademliaAction::StartBootstrap { key: my_id });
                        }
                    }
                    Err(e) => bug_condition!("p2p discovery error: {:?}", e),
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "p2p-libp2p")]
impl P2pState {
    fn p2p_pnet_timeouts<State, Action>(
        &self,
        dispatcher: &mut Dispatcher<Action, State>,
        time: Timestamp,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let timeouts = &self.config.timeouts;

        self.network
            .scheduler
            .connections
            .iter()
            .filter(|(_, state)| state.pnet.is_timed_out(time, timeouts))
            .map(|(addr, _)| P2pNetworkPnetAction::Timeout { addr: *addr })
            .for_each(|action| dispatcher.push(action));

        Ok(())
    }

    fn p2p_select_timeouts<State, Action>(
        &self,
        dispatcher: &mut Dispatcher<Action, State>,
        time: Timestamp,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let timeouts = &self.config.timeouts;

        self.network
            .scheduler
            .connections
            .iter()
            .filter(|(_, state)| state.select_auth.is_timed_out(time, timeouts))
            .map(|(addr, _)| P2pNetworkSelectAction::Timeout {
                addr: *addr,
                kind: crate::SelectKind::Authentication,
            })
            .for_each(|action| dispatcher.push(action));

        self.network
            .scheduler
            .connections
            .iter()
            .filter(|(_, state)| state.select_mux.is_timed_out(time, timeouts))
            .map(|(addr, _)| P2pNetworkSelectAction::Timeout {
                addr: *addr,
                kind: crate::SelectKind::MultiplexingNoPeerId,
            })
            .for_each(|action| dispatcher.push(action));

        // TODO: better solution for PeerId
        let dummy = PeerId::from_bytes([0u8; 32]);
        self.network
            .scheduler
            .connections
            .iter()
            .flat_map(|(sock_addr, state)| {
                state
                    .streams
                    .iter()
                    .filter(|(_, stream)| stream.select.is_timed_out(time, timeouts))
                    .map(|(stream_id, _)| (*sock_addr, *stream_id))
            })
            .map(|(addr, stream_id)| P2pNetworkSelectAction::Timeout {
                addr,
                kind: crate::SelectKind::Stream(dummy, stream_id),
            })
            .for_each(|action| dispatcher.push(action));

        Ok(())
    }

    fn p2p_rpc_heartbeats<State, Action>(
        &self,
        dispatcher: &mut Dispatcher<Action, State>,
        time: Timestamp,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let scheduler = &self.network.scheduler;

        scheduler
            .rpc_incoming_streams
            .iter()
            .chain(&scheduler.rpc_outgoing_streams)
            .flat_map(|(peer_id, state)| {
                state
                    .iter()
                    .filter(|(_, s)| s.should_send_heartbeat(time))
                    .map(|(stream_id, state)| P2pNetworkRpcAction::HeartbeatSend {
                        addr: state.addr,
                        peer_id: *peer_id,
                        stream_id: *stream_id,
                    })
            })
            .for_each(|action| dispatcher.push(action));

        Ok(())
    }
}

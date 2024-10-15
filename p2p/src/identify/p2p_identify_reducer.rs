use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    token::{BroadcastAlgorithm, DiscoveryAlgorithm, IdentifyAlgorithm, RpcAlgorithm, StreamKind},
    P2pNetworkConnectionMuxState, P2pNetworkKadRequestAction, P2pNetworkKadState,
    P2pNetworkKademliaAction, P2pNetworkYamuxAction, P2pState, YamuxStreamKind,
};

use super::P2pIdentifyAction;

impl P2pState {
    #[cfg(feature = "p2p-libp2p")]
    pub fn identify_reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<P2pIdentifyAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, _meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;

        match action {
            P2pIdentifyAction::NewRequest { addr, .. } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                let scheduler = &p2p_state.network.scheduler;
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
                    })?;

                dispatcher.push(P2pNetworkYamuxAction::OpenStream {
                    addr,
                    stream_id,
                    stream_kind: StreamKind::Identify(IdentifyAlgorithm::Identify1_0_0),
                });

                Ok(())
            }
            P2pIdentifyAction::UpdatePeerInformation {
                peer_id,
                info,
                addr,
            } => {
                let info = *info;
                if let Some(peer) = p2p_state.peers.get_mut(&peer_id) {
                    peer.identify = Some(info.clone());
                } else {
                    bug_condition!(
                        "Peer state not found for `P2pIdentifyAction::UpdatePeerInformation`"
                    );
                    return Ok(());
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                dispatcher.push(P2pNetworkKademliaAction::UpdateRoutingTable {
                    peer_id,
                    addrs: info.listen_addrs,
                });

                let stream_id = YamuxStreamKind::Rpc.stream_id(addr.incoming);

                let stream_kind = StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1);
                if !info.protocols.contains(&stream_kind) {
                    dispatcher.push(P2pDisconnectionAction::Init {
                        peer_id,
                        reason: P2pDisconnectionReason::Unsupported,
                    });
                    return Ok(());
                }

                {
                    let state: &P2pState = state.substate()?;
                    state.channels_init(dispatcher, peer_id);
                }

                dispatcher.push(P2pNetworkYamuxAction::OpenStream {
                    addr,
                    stream_id,
                    stream_kind,
                });

                let stream_kind = StreamKind::Broadcast(BroadcastAlgorithm::Meshsub1_1_0);
                if info.protocols.contains(&stream_kind) {
                    dispatcher.push(P2pNetworkYamuxAction::OpenStream {
                        addr,
                        stream_id: stream_id + 2,
                        stream_kind,
                    });
                }

                let kad_state: Option<&P2pNetworkKadState> = state.substate().ok();
                let protocol = StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0);
                if kad_state.map_or(false, |state| state.request(&peer_id).is_some())
                    && info.protocols.contains(&protocol)
                {
                    dispatcher.push(P2pNetworkKadRequestAction::MuxReady { peer_id, addr });
                }

                Ok(())
            }
        }
    }
}

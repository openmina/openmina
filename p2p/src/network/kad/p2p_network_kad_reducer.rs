use crate::{P2pLimits, P2pNetworkKadEntry};
use openmina_core::{debug, Substate, SubstateAccess};
use redux::ActionWithMeta;

use super::{
    bootstrap::P2pNetworkKadBootstrapState,
    request::P2pNetworkKadRequestState,
    stream::{P2pNetworkKadStreamState, P2pNetworkKademliaStreamAction},
    P2pNetworkKadAction, P2pNetworkKadBootstrapAction, P2pNetworkKadKey,
    P2pNetworkKadLatestRequestPeerKind, P2pNetworkKadRequestAction, P2pNetworkKadState,
    P2pNetworkKadStatus, P2pNetworkKademliaAction, P2pNetworkKademliaRpcReply,
};

impl super::P2pNetworkKadState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, Self>,
        action: ActionWithMeta<P2pNetworkKadAction>,
        limits: &P2pLimits,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let state = state_context.get_substate_mut()?;
        let (action, meta) = action.split();
        match action {
            P2pNetworkKadAction::System(action) => P2pNetworkKadState::system_reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(action),
            ),
            P2pNetworkKadAction::Bootstrap(action) => {
                let filter_addrs = state.filter_addrs;
                P2pNetworkKadBootstrapState::reducer(
                    Substate::from_compatible_substate(state_context),
                    meta.with_action(action),
                    filter_addrs,
                )
            }
            P2pNetworkKadAction::Request(request) => {
                P2pNetworkKadRequestState::reducer(state_context, meta.with_action(request))
            }
            P2pNetworkKadAction::Stream(action) => {
                P2pNetworkKadStreamState::reducer(state_context, meta.with_action(action), limits)
            }
        }
    }

    pub fn system_reducer<State, Action>(
        mut state_context: Substate<Action, State, Self>,
        action: ActionWithMeta<P2pNetworkKademliaAction>,
    ) -> Result<(), String>
    where
        State: SubstateAccess<Self>,
        Action: crate::P2pActionTrait<State>,
    {
        let state = state_context.get_substate_mut()?;

        let (action, meta) = action.split();
        match (&mut state.status, action) {
            (
                _,
                P2pNetworkKademliaAction::AnswerFindNodeRequest {
                    addr,
                    peer_id,
                    stream_id,
                    key,
                },
            ) => {
                let kad_key = P2pNetworkKadKey::from(key);
                let closer_peers: Vec<_> =
                    state.routing_table.find_node(&kad_key).cloned().collect();
                debug!(meta.time(); "found {} peers", closer_peers.len());
                let message = P2pNetworkKademliaRpcReply::FindNode { closer_peers };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkKademliaStreamAction::SendResponse {
                    addr,
                    peer_id,
                    stream_id,
                    data: message,
                });
                Ok(())
            }
            (
                _,
                P2pNetworkKademliaAction::UpdateFindNodeRequest {
                    closest_peers,
                    peer_id,
                    stream_id,
                    ..
                },
            ) => {
                let mut latest_request_peers = Vec::new();
                for entry in &closest_peers {
                    let kind = match state.routing_table.insert(entry.clone()) {
                        Ok(true) => P2pNetworkKadLatestRequestPeerKind::New,
                        Ok(false) => P2pNetworkKadLatestRequestPeerKind::Existing,
                        Err(_) => P2pNetworkKadLatestRequestPeerKind::Discarded,
                    };
                    latest_request_peers.push((entry.peer_id, kind));
                }
                state.latest_request_peers = latest_request_peers.into();

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkKadRequestAction::ReplyReceived {
                    peer_id,
                    stream_id,
                    data: closest_peers,
                });

                Ok(())
            }
            (_, P2pNetworkKademliaAction::StartBootstrap { key }) => {
                state.status = P2pNetworkKadStatus::Bootstrapping(
                    P2pNetworkKadBootstrapState::new(key).map_err(|k| k.to_string())?,
                );

                if state.bootstrap_state().map_or(false, |bootstrap_state| {
                    bootstrap_state.requests.len() < super::ALPHA
                }) {
                    let dispatcher = state_context.into_dispatcher();
                    dispatcher.push(P2pNetworkKadBootstrapAction::CreateRequests);
                }

                Ok(())
            }
            (
                P2pNetworkKadStatus::Bootstrapping(bootstrap_state),
                P2pNetworkKademliaAction::BootstrapFinished {},
            ) => {
                state.status = P2pNetworkKadStatus::Bootstrapped {
                    time: meta.time(),
                    stats: bootstrap_state.stats.clone(),
                };
                Ok(())
            }
            (_, P2pNetworkKademliaAction::UpdateRoutingTable { peer_id, addrs }) => {
                let _ = state.routing_table.insert(
                    P2pNetworkKadEntry::new(peer_id, addrs.clone()).map_err(|e| e.to_string())?,
                );
                Ok(())
            }
            (state, action) => Err(format!("invalid action {action:?} for state {state:?}")),
        }
    }
}

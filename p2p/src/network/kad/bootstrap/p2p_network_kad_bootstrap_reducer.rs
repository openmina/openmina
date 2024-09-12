use multiaddr::Multiaddr;
use openmina_core::{bug_condition, Substate, SubstateAccess};
use redux::{ActionWithMeta, Timestamp};

use crate::{
    bootstrap::{
        P2pNetworkKadBoostrapRequestState, P2pNetworkKadBootstrapFailedRequest,
        P2pNetworkKadBootstrapOngoingRequest, P2pNetworkKadBootstrapRequestStat,
        P2pNetworkKadBootstrapSuccessfulRequest,
    },
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    socket_addr_try_from_multiaddr, P2pNetworkKadEntry, P2pNetworkKadRequestAction,
    P2pNetworkKadState, P2pNetworkKademliaAction, P2pState,
};

use super::{P2pNetworkKadBootstrapAction, P2pNetworkKadBootstrapState};

fn prepare_next_request(
    addrs: &[Multiaddr],
    time: Timestamp,
    filter_addrs: bool,
) -> Option<P2pNetworkKadBoostrapRequestState> {
    let mut addrs = addrs
        .iter()
        .map(socket_addr_try_from_multiaddr)
        .filter_map(Result::ok)
        // TODO(akoptelov): remove this filtering when multiple address support is added
        .filter(|addr| {
            !filter_addrs
                || match addr.ip() {
                    std::net::IpAddr::V4(ipv4) if ipv4.is_loopback() || ipv4.is_private() => false,
                    std::net::IpAddr::V6(ipv6) if ipv6.is_loopback() => false,
                    _ => true,
                }
        });

    let addr = addrs.next()?;
    let addrs_to_use = addrs.collect();
    Some(P2pNetworkKadBoostrapRequestState {
        addr,
        time,
        addrs_to_use,
    })
}

impl P2pNetworkKadBootstrapState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pNetworkKadBootstrapAction>,
        filter_addrs: bool,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();

        match action {
            P2pNetworkKadBootstrapAction::CreateRequests => {
                let discovery_state: &P2pNetworkKadState =
                    state_context.get_substate()?.substate()?;
                let routing_table = &discovery_state.routing_table;
                let bootstrap_state: &Self = state_context.get_substate()?.substate()?;

                let requests_to_create = 3_usize.saturating_sub(bootstrap_state.requests.len());
                let peer_id_req_vec = routing_table
                    .closest_peers(&bootstrap_state.kademlia_key) // for the next request we take closest peer
                    .filter(|entry| !bootstrap_state.processed_peers.contains(&entry.peer_id)) // that is not yet processed during this bootstrap
                    .filter_map(|P2pNetworkKadEntry { peer_id, addrs, .. }| {
                        // we create a request for it
                        prepare_next_request(addrs, meta.time(), filter_addrs)
                            .map(|req| (*peer_id, req))
                    })
                    .take(requests_to_create) // and stop when we create enough requests so up to 3 will be executed in parallel
                    .collect::<Vec<_>>();

                let state: &mut Self = state_context.get_substate_mut()?.substate_mut()?;
                for (peer_id, request) in peer_id_req_vec {
                    state.processed_peers.insert(peer_id);
                    let address =
                        P2pConnectionOutgoingInitOpts::LibP2P((peer_id, request.addr).into());
                    state
                        .stats
                        .requests
                        .push(P2pNetworkKadBootstrapRequestStat::Ongoing(
                            P2pNetworkKadBootstrapOngoingRequest {
                                peer_id,
                                address,
                                start: meta.time(),
                            },
                        ));
                    state.requests.insert(peer_id, request);
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let bootstrap_state: &P2pNetworkKadBootstrapState = state.substate()?;
                let discovery_state: &P2pNetworkKadState = state.substate()?;
                let key = bootstrap_state.key;

                if bootstrap_state.requests.is_empty() {
                    dispatcher.push(P2pNetworkKademliaAction::BootstrapFinished {});
                } else {
                    bootstrap_state
                        .requests
                        .iter()
                        .filter_map(|(peer_id, req)| {
                            (!discovery_state.requests.contains_key(peer_id)).then_some(
                                P2pNetworkKadRequestAction::New {
                                    peer_id: *peer_id,
                                    addr: req.addr,
                                    key,
                                },
                            )
                        })
                        .for_each(|action| dispatcher.push(action));
                }

                Ok(())
            }
            P2pNetworkKadBootstrapAction::RequestDone {
                peer_id,
                closest_peers,
            } => {
                let state: &mut P2pNetworkKadBootstrapState =
                    state_context.get_substate_mut()?.substate_mut()?;
                let Some(req) = state.requests.remove(peer_id) else {
                    bug_condition!("cannot find request for peer {peer_id}");
                    return Ok(());
                };
                state.successful_requests += 1;
                let address = P2pConnectionOutgoingInitOpts::LibP2P((*peer_id, req.addr).into());

                if let Some(request_stats) =
                    state.stats.requests.iter_mut().rev().find(|req_stat| {
                        matches!(
                            req_stat,
                            P2pNetworkKadBootstrapRequestStat::Ongoing(
                                P2pNetworkKadBootstrapOngoingRequest {
                                    address: a,
                                    ..
                                },
                            ) if a == &address
                        )
                    })
                {
                    *request_stats = P2pNetworkKadBootstrapRequestStat::Successful(
                        P2pNetworkKadBootstrapSuccessfulRequest {
                            peer_id: *peer_id,
                            address,
                            start: req.time,
                            finish: meta.time(),
                            closest_peers: closest_peers.clone(),
                        },
                    );
                } else {
                    bug_condition!("cannot find stats for request {req:?}");
                };

                if state.successful_requests < 20 {
                    let dispatcher = state_context.into_dispatcher();
                    dispatcher.push(P2pNetworkKadBootstrapAction::CreateRequests);
                }

                Ok(())
            }
            P2pNetworkKadBootstrapAction::RequestError { peer_id, error } => {
                let bootstrap_state: &mut P2pNetworkKadBootstrapState =
                    state_context.get_substate_mut()?.substate_mut()?;

                let Some(req) = bootstrap_state.requests.remove(peer_id) else {
                    bug_condition!("cannot find request for peer {peer_id}");
                    return Ok(());
                };

                let address = P2pConnectionOutgoingInitOpts::LibP2P((*peer_id, req.addr).into());
                if let Some(request_stats) =
                    bootstrap_state
                        .stats
                        .requests
                        .iter_mut()
                        .rev()
                        .find(|req_stat| {
                            matches!(
                                req_stat,
                                P2pNetworkKadBootstrapRequestStat::Ongoing(
                                    P2pNetworkKadBootstrapOngoingRequest {
                                        address: a,
                                        ..
                                    },
                                ) if a == &address
                            )
                        })
                {
                    *request_stats = P2pNetworkKadBootstrapRequestStat::Failed(
                        P2pNetworkKadBootstrapFailedRequest {
                            peer_id: *peer_id,
                            address,
                            start: req.time,
                            finish: meta.time(),
                            error: error.clone(),
                        },
                    );
                } else {
                    bug_condition!("cannot find stats for request {req:?}");
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkKadBootstrapAction::CreateRequests);
                Ok(())
            }
        }
    }
}

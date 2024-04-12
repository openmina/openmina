use redux::{ActionMeta, ActionWithMeta};

use crate::{
    channels::{P2pChannelsAction, P2pChannelsService},
    connection::{
        outgoing::P2pConnectionOutgoingAction, P2pConnectionAction, P2pConnectionService,
    },
    disconnection::P2pDisconnectionService,
    discovery::P2pDiscoveryAction,
    P2pAction, P2pCryptoService, P2pMioService, P2pStore,
};

pub fn p2p_timeout_effects<Store, S>(store: &mut Store, meta: &ActionMeta)
where
    Store: P2pStore<S>,
{
    p2p_connection_timeouts(store, meta);
    store.dispatch(P2pConnectionOutgoingAction::RandomInit);

    p2p_try_reconnect_disconnected_peers(store, meta.time());

    store.dispatch(P2pDiscoveryAction::KademliaBootstrap);
    store.dispatch(P2pDiscoveryAction::KademliaInit);

    #[cfg(feature = "p2p-webrtc")]
    p2p_discovery_request(store, meta);

    #[cfg(not(feature = "p2p-libp2p"))]
    store.dispatch(
        crate::network::kad::P2pNetworkKademliaAction::StartBootstrap {
            key: store.state().config.identity_pub_key.peer_id(),
        },
    );

    let state = store.state();
    for (peer_id, id) in state.peer_rpc_timeouts(meta.time()) {
        store.dispatch(crate::channels::rpc::P2pChannelsRpcAction::Timeout { peer_id, id });
    }
}

fn p2p_connection_timeouts<Store, S>(store: &mut Store, meta: &ActionMeta)
where
    Store: P2pStore<S>,
{
    use crate::connection::incoming::P2pConnectionIncomingAction;

    let now = meta.time();
    let timeouts = &store.state().config.timeouts;
    let p2p_connection_timeouts: Vec<_> = store
        .state()
        .peers
        .iter()
        .filter_map(|(peer_id, peer)| {
            let s = peer.status.as_connecting()?;
            match s.is_timed_out(now, timeouts) {
                true => Some((*peer_id, s.as_outgoing().is_some())),
                false => None,
            }
        })
        .collect();

    for (peer_id, is_outgoing) in p2p_connection_timeouts {
        match is_outgoing {
            true => store.dispatch(P2pConnectionOutgoingAction::Timeout { peer_id }),
            false => store.dispatch(P2pConnectionIncomingAction::Timeout { peer_id }),
        };
    }
}

fn p2p_try_reconnect_disconnected_peers<Store, S>(store: &mut Store, now: redux::Timestamp)
where
    Store: P2pStore<S>,
{
    if store.state().already_has_min_peers() {
        return;
    }
    let timeouts = &store.state().config.timeouts;
    let reconnect_actions: Vec<_> = store
        .state()
        .peers
        .iter()
        .filter_map(|(_, p)| {
            if p.can_reconnect(now, timeouts) {
                p.dial_opts.clone()
            } else {
                None
            }
        })
        .map(|opts| P2pConnectionOutgoingAction::Reconnect { opts, rpc_id: None })
        .collect();
    for action in reconnect_actions {
        store.dispatch(action);
    }
}

/// Iterate all connected peers and check the time of the last response to the peer discovery request.
/// If the elapsed time is large enough, send another discovery request.
#[cfg(feature = "p2p-webrtc")]
fn p2p_discovery_request<Store, S>(store: &mut Store, meta: &redux::ActionMeta)
where
    Store: P2pStore<S>,
{
    let peer_ids = store
        .state()
        .ready_peers_iter()
        .filter_map(|(peer_id, _)| {
            let Some(t) = store.state().kademlia.peer_timestamp.get(peer_id).cloned() else {
                return Some(*peer_id);
            };
            let elapsed = meta
                .time_as_nanos()
                .checked_sub(t.into())
                .unwrap_or_default();
            let minimal_interval = store.state().config.ask_initial_peers_interval;
            if elapsed < minimal_interval.as_nanos() as u64 {
                None
            } else {
                Some(*peer_id)
            }
        })
        .collect::<Vec<_>>();

    for peer_id in peer_ids {
        store.dispatch(P2pDiscoveryAction::Init { peer_id });
    }
}

pub fn p2p_effects<Store, S>(store: &mut Store, action: ActionWithMeta<P2pAction>)
where
    Store: P2pStore<S>,
    Store::Service: P2pConnectionService
        + P2pDisconnectionService
        + P2pChannelsService
        + P2pMioService
        + P2pCryptoService,
{
    let (action, meta) = action.split();
    match action {
        P2pAction::Listen(action) => action.effects(&meta, store),
        P2pAction::Connection(action) => match action {
            P2pConnectionAction::Outgoing(action) => action.effects(&meta, store),
            P2pConnectionAction::Incoming(action) => action.effects(&meta, store),
        },
        P2pAction::Disconnection(action) => action.effects(&meta, store),
        P2pAction::Discovery(action) => action.effects(&meta, store),
        P2pAction::Channels(action) => match action {
            P2pChannelsAction::MessageReceived(action) => action.effects(&meta, store),
            P2pChannelsAction::BestTip(action) => action.effects(&meta, store),
            P2pChannelsAction::Snark(action) => action.effects(&meta, store),
            P2pChannelsAction::SnarkJobCommitment(action) => action.effects(&meta, store),
            P2pChannelsAction::Rpc(action) => action.effects(&meta, store),
        },
        P2pAction::Peer(action) => action.effects(&meta, store),
        P2pAction::Network(action) => action.effects(&meta, store),
    }
}

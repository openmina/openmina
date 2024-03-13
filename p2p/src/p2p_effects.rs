use redux::{ActionMeta, ActionWithMeta};

use crate::{
    channels::{P2pChannelsAction, P2pChannelsService},
    connection::{
        outgoing::P2pConnectionOutgoingAction, P2pConnectionAction, P2pConnectionService,
    },
    disconnection::P2pDisconnectionService,
    is_time_passed, P2pAction, P2pCryptoService, P2pMioService, P2pNetworkKadStatus,
    P2pNetworkKademliaAction, P2pStore,
};

pub fn p2p_timeout_effects<Store, S>(store: &mut Store, meta: &ActionMeta)
where
    Store: P2pStore<S>,
{
    p2p_connection_timeouts(store, meta);
    store.dispatch(P2pConnectionOutgoingAction::RandomInit);

    p2p_try_reconnect_disconnected_peers(store, meta.time());

    p2p_discovery(store, meta);

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

fn p2p_discovery<Store, S>(store: &mut Store, meta: &redux::ActionMeta)
where
    Store: P2pStore<S>,
{
    let now = meta.time();
    let state = store.state();
    let config = &state.config;
    if !config.peer_discovery {
        return;
    }
    // ask initial peers
    if let Some(_d) = config.timeouts.initial_peers {
        // TODO: use RPC to ask initial peers
    }

    if let Some(discovery_state) = state.network.scheduler.discovery_state() {
        let bootstrap_kademlia = match &discovery_state.status {
            P2pNetworkKadStatus::Init => true,
            P2pNetworkKadStatus::Bootstrapping(_) => false,
            P2pNetworkKadStatus::Bootstrapped { time, .. } => {
                is_time_passed(now, *time, config.timeouts.kademlia_bootstrap)
            }
        };
        if bootstrap_kademlia {
            store.dispatch(P2pNetworkKademliaAction::StartBootstrap {
                key: config.identity_pub_key.peer_id(),
            });
        }
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
        P2pAction::Connection(action) => match action {
            P2pConnectionAction::Outgoing(action) => action.effects(&meta, store),
            P2pConnectionAction::Incoming(action) => action.effects(&meta, store),
        },
        P2pAction::Disconnection(action) => action.effects(&meta, store),
        P2pAction::Discovery(action) => action.effects(&meta, store),
        P2pAction::Identify(action) => action.effects(&meta, store),
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

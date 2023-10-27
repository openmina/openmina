use p2p::{connection::outgoing::P2pConnectionOutgoingInitOpts, P2pDiscoveryEvent, P2pEvent};

use crate::{event_source::Event, Action, ActionWithMeta, EventSourceAction, State};

pub fn reducer(state: &mut State, action: &ActionWithMeta) {
    let meta = action.meta().clone();
    match action.action() {
        Action::CheckTimeouts(_) => {}
        Action::EventSource(EventSourceAction::NewEvent(content)) => match &content.event {
            Event::P2p(P2pEvent::Libp2pIdentify(peer_id, maddr)) => {
                if let Some(peer) = state.p2p.peers.get_mut(peer_id) {
                    peer.dial_opts = Some(P2pConnectionOutgoingInitOpts::LibP2P {
                        peer_id: *peer_id,
                        maddr: maddr.clone(),
                    });
                }
            }
            Event::P2p(P2pEvent::Discovery(P2pDiscoveryEvent::Ready)) => {
                state.p2p.kademlia.is_ready = true;
            }
            Event::P2p(P2pEvent::Discovery(P2pDiscoveryEvent::DidFindPeers(optses))) => {
                for opts in optses {
                    state.p2p.known_peers.insert(*opts.peer_id(), opts.clone());
                }
            }
            _ => {}
        },
        Action::EventSource(_) => {}

        Action::P2p(a) => {
            state.p2p.reducer(meta.with_action(a));
        }
        Action::Snark(a) => {
            state.snark.reducer(meta.with_action(a));
        }
        Action::Consensus(a) => {
            state.consensus.reducer(meta.with_action(a));
        }
        Action::TransitionFrontier(a) => {
            state.transition_frontier.reducer(meta.with_action(a));
        }
        Action::SnarkPool(a) => {
            state.snark_pool.reducer(meta.with_action(a));
        }
        Action::Rpc(a) => {
            state.rpc.reducer(meta.with_action(a));
        }
        Action::ExternalSnarkWorker(a) => {
            state.external_snark_worker.reducer(meta.with_action(a));
        }
        Action::WatchedAccounts(a) => {
            state.watched_accounts.reducer(meta.with_action(a));
        }
    }

    // must be the last.
    state.action_applied(action);
}

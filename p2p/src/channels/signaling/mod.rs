//! There are 2 state machines in this module:
//! 1. `discovery` - used for discovering a new target peer and initiating
//!    signaling process.
//! 2. `exchange` - used by intermediary peer to relay an offer to the
//!    target peer and receive an answer from it.
//!
//! These are the overall steps that happens in these state machines in
//! order to connect two (dialer and listener) peers to each other using
//! intermediary peer (relayer):
//! 1. [discovery] Dialer asks relayer to discover an available peer.
//! 2. [discovery] Relayer responds with available peer's (listener's) public key.
//! 3. [discovery] Dialer accepts/rejects the target peer (listener).
//! 4. [discovery] If dialer accepts the peer, it sends webrtc offer to relayer.
//! 5. [exchange] Relayer relays received webrtc offer to the listener peer.
//! 6. [exchange] Relayer receives webrtc answer from the listener peer.
//! 7. [discovery] Relayer relays the answer to the dialer.

pub mod discovery;
pub mod discovery_effectful;
pub mod exchange;
pub mod exchange_effectful;

mod p2p_channels_signaling_state;
pub use p2p_channels_signaling_state::*;

use std::collections::BTreeSet;

use discovery::P2pChannelsSignalingDiscoveryAction;

impl crate::P2pState {
    pub(super) fn webrtc_discovery_respond_with_availble_peers<Action, State>(
        &self,
        dispatcher: &mut redux::Dispatcher<Action, State>,
    ) where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (mut available_peers, requests) = self.ready_peers_iter().fold(
            (BTreeSet::new(), BTreeSet::new()),
            |(mut available, mut requests), (peer_id, peer)| {
                if peer.channels.signaling.is_looking_for_incoming_peer() {
                    available.insert(peer_id);
                }
                if peer.channels.signaling.is_looking_for_peer() {
                    requests.insert(peer_id);
                } else if let Some(peer_id) = peer.channels.signaling.sent_discovered_peer_id() {
                    available.remove(&peer_id);
                }
                (available, requests)
            },
        );

        // TODO(binier): maybe randomize
        for requester in requests {
            if let Some(target_peer_id) =
                available_peers.iter().filter(|&&id| id != requester).next()
            {
                let target_peer_id = *target_peer_id;
                dispatcher.push(P2pChannelsSignalingDiscoveryAction::DiscoveredSend {
                    peer_id: *requester,
                    target_public_key: target_peer_id.to_public_key().unwrap(),
                });
                available_peers.remove(&target_peer_id);
            } else {
                break;
            }
        }
    }
}

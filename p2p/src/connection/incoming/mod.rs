mod p2p_connection_incoming_state;
pub use p2p_connection_incoming_state::*;

mod p2p_connection_incoming_actions;
pub use p2p_connection_incoming_actions::*;

mod p2p_connection_incoming_reducer;
pub use p2p_connection_incoming_reducer::*;

mod p2p_connection_incoming_effects;
pub use p2p_connection_incoming_effects::*;

use serde::{Deserialize, Serialize};

use crate::connection::RejectionReason;
use crate::{webrtc, P2pState, PeerId};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct P2pConnectionIncomingInitOpts {
    pub peer_id: PeerId,
    pub signaling: IncomingSignalingMethod,
    pub offer: webrtc::Offer,
}

// TODO(binier): maybe move to `crate::webrtc`?
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum IncomingSignalingMethod {
    Http,
}

impl P2pState {
    pub fn incoming_accept(
        &self,
        peer_id: PeerId,
        offer: &webrtc::Offer,
    ) -> Result<(), RejectionReason> {
        if peer_id != offer.identity_pub_key.peer_id() {
            return Err(RejectionReason::PeerIdAndPublicKeyMismatch);
        }

        let my_peer_id = self.config.identity_pub_key.peer_id();

        // TODO(binier): maybe cache own peer_id somewhere.
        if offer.target_peer_id != my_peer_id {
            return Err(RejectionReason::TargetPeerIdNotMe);
        }

        if peer_id == my_peer_id {
            return Err(RejectionReason::ConnectingToSelf);
        }

        if self.is_peer_connected_or_connecting(&peer_id) {
            return Err(RejectionReason::AlreadyConnected);
        }

        if self.already_has_max_peers() {
            return Err(RejectionReason::PeerCapacityFull);
        }

        Ok(())
    }

    pub fn libp2p_incoming_accept(&self, peer_id: PeerId) -> Result<(), RejectionReason> {
        let my_peer_id = self.config.identity_pub_key.peer_id();

        if peer_id == my_peer_id {
            return Err(RejectionReason::ConnectingToSelf);
        }

        if self.already_has_max_peers() {
            return Err(RejectionReason::PeerCapacityFull);
        }

        Ok(())
    }
}

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::{
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionEffectfulAction},
    identity::PublicKey,
    webrtc::{self, ConnectionAuth, SignalingMethod},
    P2pState, PeerId,
};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(opts), display(peer_id), display(error)))]
pub enum P2pConnectionOutgoingEffectfulAction {
    /// Initialize connection to a random peer.
    #[action_event(level = trace)]
    RandomInit {
        peers: Vec<P2pConnectionOutgoingInitOpts>,
    },
    /// Initialize connection to a new peer.
    #[action_event(level = info)]
    Init {
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    OfferSend {
        peer_id: PeerId,
        offer: Box<webrtc::Offer>,
        signaling_method: SignalingMethod,
    },
    AnswerSet {
        peer_id: PeerId,
        answer: Box<webrtc::Answer>,
    },
    ConnectionAuthorizationEncryptAndSend {
        peer_id: PeerId,
        other_pub_key: PublicKey,
        auth: ConnectionAuth,
    },
    ConnectionAuthorizationDecryptAndCheck {
        peer_id: PeerId,
        other_pub_key: PublicKey,
        expected_auth: ConnectionAuth,
        auth: webrtc::ConnectionAuthEncrypted,
    },
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingEffectfulAction {
    fn is_enabled(&self, _: &P2pState, _: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pConnectionOutgoingEffectfulAction> for crate::P2pEffectfulAction {
    fn from(a: P2pConnectionOutgoingEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Connection(P2pConnectionEffectfulAction::Outgoing(a))
    }
}

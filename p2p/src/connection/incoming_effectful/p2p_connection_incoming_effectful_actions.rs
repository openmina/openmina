use crate::{
    connection::{incoming::P2pConnectionIncomingInitOpts, P2pConnectionEffectfulAction},
    identity::PublicKey,
    webrtc::{ConnectionAuth, ConnectionAuthEncrypted},
    P2pState, PeerId,
};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(debug(opts), display(peer_id), display(error)))]
pub enum P2pConnectionIncomingEffectfulAction {
    /// Incoming connection is initialized.
    Init { opts: P2pConnectionIncomingInitOpts },
    ConnectionAuthorizationEncryptAndSend {
        peer_id: PeerId,
        other_pub_key: PublicKey,
        auth: ConnectionAuth,
    },
    ConnectionAuthorizationDecryptAndCheck {
        peer_id: PeerId,
        other_pub_key: PublicKey,
        expected_auth: ConnectionAuth,
        auth: ConnectionAuthEncrypted,
    },
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingEffectfulAction {
    fn is_enabled(&self, _: &P2pState, _: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pConnectionIncomingEffectfulAction> for crate::P2pEffectfulAction {
    fn from(a: P2pConnectionIncomingEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Connection(P2pConnectionEffectfulAction::Incoming(a))
    }
}

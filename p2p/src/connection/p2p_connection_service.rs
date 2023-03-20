use crate::{webrtc, PeerId};

use super::outgoing::P2pConnectionOutgoingInitOpts;

pub trait P2pConnectionService: redux::Service {
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts;

    /// Initiates outgoing connection and creates an offer sdp, which
    /// will be received in state machine as an event.
    fn outgoing_init(&mut self, peer_id: PeerId);

    fn set_offer(&mut self, offer: webrtc::Offer);

    fn set_answer(&mut self, answer: webrtc::Answer);

    fn http_signal_send(&mut self, url: String, signal: webrtc::Signal);
}

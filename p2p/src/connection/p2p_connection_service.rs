use crate::{webrtc, PeerId};

use super::outgoing::P2pConnectionOutgoingInitOpts;

pub trait P2pConnectionService: redux::Service {
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> Option<P2pConnectionOutgoingInitOpts>;

    /// Initiates an outgoing connection and creates an offer sdp,
    /// which will be received in the state machine as an event.
    fn outgoing_init(&mut self, opts: P2pConnectionOutgoingInitOpts);

    /// Initiates an incoming connection and creates an answer sdp,
    /// which will be received in the state machine as an event.
    fn incoming_init(&mut self, peer_id: PeerId, offer: webrtc::Offer);

    fn set_answer(&mut self, peer_id: PeerId, answer: webrtc::Answer);

    fn http_signaling_request(&mut self, url: String, offer: webrtc::Offer);
}

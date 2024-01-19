use crate::{
    webrtc::{self, SignalingMethod},
    PeerId,
};

pub trait P2pConnectionWebRTCService: redux::Service {
    /// Initiates an outgoing connection and creates an offer sdp,
    /// which will be received in the state machine as an event.
    fn outgoing_init(&mut self, peer_id: PeerId, opts: SignalingMethod);

    /// Initiates an incoming connection and creates an answer sdp,
    /// which will be received in the state machine as an event.
    fn incoming_init(&mut self, peer_id: PeerId, offer: webrtc::Offer);

    fn set_answer(&mut self, peer_id: PeerId, answer: webrtc::Answer);

    fn http_signaling_request(&mut self, url: String, offer: webrtc::Offer);

    // fn find_random_peer(&mut self);
}

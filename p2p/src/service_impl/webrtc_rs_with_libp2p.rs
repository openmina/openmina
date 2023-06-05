use libp2p::swarm::dial_opts::DialOpts;
use tokio::sync::mpsc;

use crate::{
    channels::{ChannelId, ChannelMsg, MsgId, P2pChannelsService},
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionService},
    disconnection::P2pDisconnectionService,
    P2pEvent, PeerId,
};

use super::{libp2p::Libp2pService, webrtc_rs::P2pServiceWebrtcRs};

pub struct P2pServiceCtx {
    pub webrtc: super::webrtc_rs::P2pServiceCtx,
    pub libp2p: Libp2pService,
}

pub trait P2pServiceWebrtcRsWithLibp2p: P2pServiceWebrtcRs {
    fn libp2p(&mut self) -> &mut Libp2pService;

    fn init(
        chain_id: String,
        event_source_sender: mpsc::UnboundedSender<P2pEvent>,
    ) -> P2pServiceCtx {
        P2pServiceCtx {
            webrtc: <Self as P2pServiceWebrtcRs>::init(),
            libp2p: Libp2pService::run(chain_id, event_source_sender),
        }
    }
}

impl<T: P2pServiceWebrtcRsWithLibp2p> P2pConnectionService for T {
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        P2pServiceWebrtcRs::random_pick(self, list)
    }

    fn outgoing_init(&mut self, opts: P2pConnectionOutgoingInitOpts) {
        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { peer_id, .. } => {
                P2pServiceWebrtcRs::outgoing_init(self, peer_id);
            }
            P2pConnectionOutgoingInitOpts::LibP2P { peer_id, maddr } => {
                let opts = DialOpts::peer_id(peer_id.into())
                    .addresses(vec![maddr])
                    .build();
                let cmd = super::libp2p::Cmd::Dial(opts);
                let _ = self.libp2p().cmd_sender().send(cmd);
            }
        }
    }

    fn incoming_init(&mut self, peer_id: PeerId, offer: crate::webrtc::Offer) {
        P2pServiceWebrtcRs::incoming_init(self, peer_id, offer)
    }

    fn set_answer(&mut self, peer_id: PeerId, answer: crate::webrtc::Answer) {
        P2pServiceWebrtcRs::set_answer(self, peer_id, answer)
    }

    fn http_signaling_request(&mut self, url: String, offer: crate::webrtc::Offer) {
        P2pServiceWebrtcRs::http_signaling_request(self, url, offer)
    }
}

impl<T: P2pServiceWebrtcRs> P2pDisconnectionService for T {
    fn disconnect(&mut self, peer_id: PeerId) {
        // By removing the peer, `cmd_sender` gets dropped which will
        // cause `peer_loop` to end.
        self.peers().remove(&peer_id);
    }
}

impl<T: P2pServiceWebrtcRs> P2pChannelsService for T {
    fn channel_open(&mut self, peer_id: PeerId, id: ChannelId) {
        // TODO(binier)
    }

    fn channel_send(&mut self, peer_id: PeerId, msg_id: MsgId, msg: ChannelMsg) {
        // TODO(binier)
    }
}

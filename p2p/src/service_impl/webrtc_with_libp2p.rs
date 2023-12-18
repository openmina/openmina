use openmina_core::channels::mpsc;
use openmina_core::snark::Snark;

use crate::{
    channels::{ChannelId, ChannelMsg, MsgId, P2pChannelsService},
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionService},
    disconnection::P2pDisconnectionService,
    identity::SecretKey,
    P2pChannelEvent, P2pEvent, PeerId,
};

use super::{libp2p::Libp2pService, webrtc::P2pServiceWebrtc, TaskSpawner};

pub struct P2pServiceCtx {
    pub webrtc: super::webrtc::P2pServiceCtx,
    pub libp2p: Libp2pService,
}

pub trait P2pServiceWebrtcWithLibp2p: P2pServiceWebrtc {
    fn libp2p(&mut self) -> &mut Libp2pService;

    fn init<S: TaskSpawner>(
        libp2p_port: Option<u16>,
        secret_key: SecretKey,
        chain_id: String,
        event_source_sender: mpsc::UnboundedSender<P2pEvent>,
        spawner: S,
    ) -> P2pServiceCtx {
        P2pServiceCtx {
            webrtc: <Self as P2pServiceWebrtc>::init(secret_key.clone(), spawner.clone()),
            libp2p: Libp2pService::run(
                libp2p_port,
                secret_key,
                chain_id,
                event_source_sender,
                spawner,
            ),
        }
    }

    fn find_random_peer(&mut self);

    fn start_discovery(&mut self, peers: Vec<P2pConnectionOutgoingInitOpts>);
}

impl<T: P2pServiceWebrtcWithLibp2p> P2pConnectionService for T {
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        P2pServiceWebrtc::random_pick(self, list)
    }

    fn outgoing_init(&mut self, opts: P2pConnectionOutgoingInitOpts) {
        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { peer_id, .. } => {
                P2pServiceWebrtc::outgoing_init(self, peer_id);
            }
            P2pConnectionOutgoingInitOpts::LibP2P(opts) => {
                let cmd = super::libp2p::Cmd::Dial(opts.peer_id.into(), vec![opts.to_maddr()]);
                let _ = self.libp2p().cmd_sender().send(cmd);
            }
        }
    }

    fn incoming_init(&mut self, peer_id: PeerId, offer: crate::webrtc::Offer) {
        P2pServiceWebrtc::incoming_init(self, peer_id, offer)
    }

    fn set_answer(&mut self, peer_id: PeerId, answer: crate::webrtc::Answer) {
        P2pServiceWebrtc::set_answer(self, peer_id, answer)
    }

    fn http_signaling_request(&mut self, url: String, offer: crate::webrtc::Offer) {
        P2pServiceWebrtc::http_signaling_request(self, url, offer)
    }

    fn start_discovery(&mut self, peers: Vec<P2pConnectionOutgoingInitOpts>) {
        P2pServiceWebrtcWithLibp2p::start_discovery(self, peers)
    }

    fn find_random_peer(&mut self) {
        P2pServiceWebrtcWithLibp2p::find_random_peer(self);
    }
}

impl<T: P2pServiceWebrtcWithLibp2p> P2pDisconnectionService for T {
    fn disconnect(&mut self, peer_id: PeerId) {
        // By removing the peer, `cmd_sender` gets dropped which will
        // cause `peer_loop` to end.
        let is_libp2p_peer = self.peers().remove(&peer_id).is_none();
        if is_libp2p_peer {
            use super::libp2p::Cmd;
            let _ = self
                .libp2p()
                .cmd_sender()
                .send(Cmd::Disconnect(peer_id.into()));
        }
    }
}

impl<T: P2pServiceWebrtcWithLibp2p> P2pChannelsService for T {
    fn channel_open(&mut self, peer_id: PeerId, id: ChannelId) {
        if self.peers().contains_key(&peer_id) {
            P2pServiceWebrtc::channel_open(self, peer_id, id)
        } else {
            let result = match id.supported_by_libp2p() {
                false => Err("channel not supported".to_owned()),
                true => Ok(()),
            };
            let _ = self
                .event_sender()
                .send(P2pEvent::Channel(P2pChannelEvent::Opened(
                    peer_id, id, result,
                )));
        }
    }

    fn channel_send(&mut self, peer_id: PeerId, msg_id: MsgId, msg: ChannelMsg) {
        if self.peers().contains_key(&peer_id) {
            P2pServiceWebrtc::channel_send(self, peer_id, msg_id, msg)
        } else {
            use super::libp2p::Cmd;
            let _ = self
                .libp2p()
                .cmd_sender()
                .send(Cmd::SendMessage(peer_id.into(), msg));
        }
    }

    fn libp2p_broadcast_snark(&mut self, snark: Snark, nonce: u32) {
        use super::libp2p::Cmd;
        let _ = self
            .libp2p()
            .cmd_sender()
            .send(Cmd::SnarkBroadcast(snark, nonce));
    }
}

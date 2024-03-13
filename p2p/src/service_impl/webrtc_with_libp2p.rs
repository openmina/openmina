use openmina_core::channels::mpsc;
use openmina_core::snark::Snark;

use crate::{
    channels::{ChannelId, ChannelMsg, MsgId, P2pChannelsService},
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionService},
    disconnection::P2pDisconnectionService,
    identity::SecretKey,
    P2pChannelEvent, P2pEvent, PeerId,
};

#[cfg(not(feature = "p2p-libp2p"))]
use super::mio::MioService;
#[cfg(not(feature = "p2p-libp2p"))]
use crate::P2pMioService;

use super::{webrtc::P2pServiceWebrtc, TaskSpawner};

pub struct P2pServiceCtx {
    pub webrtc: super::webrtc::P2pServiceCtx,
    #[cfg(feature = "p2p-libp2p")]
    pub libp2p: super::libp2p::Libp2pService,
    #[cfg(not(feature = "p2p-libp2p"))]
    pub mio: MioService,
}

pub trait P2pServiceWebrtcWithLibp2p: P2pServiceWebrtc {
    #[cfg(feature = "p2p-libp2p")]
    fn libp2p(&mut self) -> &mut super::libp2p::Libp2pService;

    #[cfg(not(feature = "p2p-libp2p"))]
    fn mio(&mut self) -> &mut MioService;

    fn init<E: From<P2pEvent> + Send + 'static, S: TaskSpawner>(
        _libp2p_port: Option<u16>,
        secret_key: SecretKey,
        _chain_id: Vec<u8>,
        event_source_sender: mpsc::UnboundedSender<E>,
        spawner: S,
    ) -> P2pServiceCtx {
        P2pServiceCtx {
            #[cfg(feature = "p2p-libp2p")]
            libp2p: super::libp2p::Libp2pService::run::<E, S>(
                _libp2p_port,
                secret_key.clone(),
                _chain_id.into(),
                event_source_sender,
                spawner.clone(),
            ),
            webrtc: <Self as P2pServiceWebrtc>::init(secret_key, spawner),
            #[cfg(not(feature = "p2p-libp2p"))]
            mio: MioService::run({
                move |mio_event| {
                    event_source_sender
                        .send(P2pEvent::MioEvent(mio_event).into())
                        .unwrap_or_default()
                }
            }),
        }
    }

    #[cfg(feature = "p2p-libp2p")]
    fn find_random_peer(&mut self);

    #[cfg(feature = "p2p-libp2p")]
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
            #[cfg(feature = "p2p-libp2p")]
            P2pConnectionOutgoingInitOpts::LibP2P(opts) => {
                let cmd = super::libp2p::Cmd::Dial(opts.peer_id.into(), vec![opts.to_maddr()]);
                let _ = self.libp2p().cmd_sender().send(cmd);
            }
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pConnectionOutgoingInitOpts::LibP2P(opts) => {
                use crate::webrtc::Host;
                let addr = match opts.host {
                    Host::Ipv4(ip4) => ip4.into(),
                    Host::Ipv6(ip6) => ip6.into(),
                    host => {
                        openmina_core::error!(openmina_core::log::system_time(); "unsupported host for internal libp2p: {host}");
                        return;
                    }
                };
                let _ = self
                    .mio()
                    .send_mio_cmd(crate::MioCmd::Connect(std::net::SocketAddr::new(
                        addr, opts.port,
                    )));
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

    #[cfg(feature = "p2p-libp2p")]
    fn start_discovery(&mut self, peers: Vec<P2pConnectionOutgoingInitOpts>) {
        P2pServiceWebrtcWithLibp2p::start_discovery(self, peers)
    }

    #[cfg(feature = "p2p-libp2p")]
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
            #[cfg(not(feature = "p2p-libp2p"))]
            {
                // TODO(akoptelov): pass dial_opt here to get IP address
                // self.mio().send_mio_cmd(MioCmd::Disconnect())
            }
            #[cfg(feature = "p2p-libp2p")]
            {
                use super::libp2p::Cmd;
                let _ = self
                    .libp2p()
                    .cmd_sender()
                    .send(Cmd::Disconnect(peer_id.into()));
            }
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
            self.event_sender()
                .send(P2pEvent::Channel(P2pChannelEvent::Opened(peer_id, id, result)).into())
                .unwrap_or_default();
        }
    }

    fn channel_send(&mut self, peer_id: PeerId, msg_id: MsgId, msg: ChannelMsg) {
        if self.peers().contains_key(&peer_id) {
            P2pServiceWebrtc::channel_send(self, peer_id, msg_id, msg)
        } else {
            #[cfg(feature = "p2p-libp2p")]
            {
                use super::libp2p::Cmd;
                let _ = self
                    .libp2p()
                    .cmd_sender()
                    .send(Cmd::SendMessage(peer_id.into(), msg));
            }
            #[cfg(not(feature = "p2p-libp2p"))]
            {
                openmina_core::error!(openmina_core::log::system_time(); "sending to channel {:?} is not supported", msg.channel_id());
            }
        }
    }

    fn libp2p_broadcast_snark(&mut self, _snark: Snark, _nonce: u32) {
        #[cfg(feature = "p2p-libp2p")]
        {
            use super::libp2p::Cmd;
            let _ = self
                .libp2p()
                .cmd_sender()
                .send(Cmd::SnarkBroadcast(_snark, _nonce));
        }
        #[cfg(not(feature = "p2p-libp2p"))]
        {
            todo!("unimplemented");
        }
    }
}

impl<T: P2pServiceWebrtcWithLibp2p> crate::P2pMioService for T {
    #[cfg(not(feature = "p2p-libp2p"))]
    fn send_mio_cmd(&mut self, cmd: crate::MioCmd) {
        self.mio().send_mio_cmd(cmd)
    }
    #[cfg(feature = "p2p-libp2p")]
    fn send_mio_cmd(&mut self, _cmd: crate::MioCmd) {
        unimplemented!("mio is not supproted")
    }
}

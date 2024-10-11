use crate::{
    channels::{ChannelId, ChannelMsg, MsgId, P2pChannelsService},
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionService},
    disconnection_effectful::P2pDisconnectionService,
    identity::SecretKey,
    P2pChannelEvent, P2pEvent, PeerId,
};

#[cfg(feature = "p2p-libp2p")]
use super::mio::MioService;
#[cfg(feature = "p2p-libp2p")]
use crate::{P2pMioService, P2pNetworkService, P2pNetworkServiceError};

use super::{webrtc::P2pServiceWebrtc, TaskSpawner};

pub struct P2pServiceCtx {
    pub sec_key: SecretKey,
    pub webrtc: super::webrtc::P2pServiceCtx,
    #[cfg(feature = "p2p-libp2p")]
    pub mio: MioService,
}

pub trait P2pServiceWebrtcWithLibp2p: P2pServiceWebrtc {
    #[cfg(feature = "p2p-libp2p")]
    fn mio(&mut self) -> &mut MioService;

    fn init<S: TaskSpawner>(sec_key: SecretKey, spawner: S) -> P2pServiceCtx {
        P2pServiceCtx {
            sec_key: sec_key.clone(),
            #[cfg(feature = "p2p-libp2p")]
            mio: MioService::pending(sec_key.clone().try_into().expect("valid keypair")),
            webrtc: <Self as P2pServiceWebrtc>::init(sec_key, spawner),
        }
    }

    #[cfg(feature = "p2p-libp2p")]
    fn resolve_name(
        &mut self,
        hostname: &str,
    ) -> Result<Vec<std::net::IpAddr>, P2pNetworkServiceError> {
        use std::net::ToSocketAddrs;

        let it = format!("{hostname}:0")
            .to_socket_addrs()
            .map_err(|err| P2pNetworkServiceError::Resolve(format!("{hostname}, {err}")))?;
        Ok(it.map(|addr| addr.ip()).collect())
    }

    #[cfg(feature = "p2p-libp2p")]
    fn detect_local_ip(&mut self) -> Result<Vec<std::net::IpAddr>, P2pNetworkServiceError> {
        let addrs = local_ip_address::list_afinet_netifas()
            .map_err(|e| P2pNetworkServiceError::LocalIp(e.to_string()))?;
        Ok(addrs.into_iter().map(|(_, ip)| ip).collect())
    }
}

#[cfg(feature = "p2p-libp2p")]
impl<T: P2pServiceWebrtcWithLibp2p> P2pNetworkService for T {
    fn resolve_name(
        &mut self,
        host: &str,
    ) -> Result<Vec<std::net::IpAddr>, P2pNetworkServiceError> {
        P2pServiceWebrtcWithLibp2p::resolve_name(self, host)
    }

    fn detect_local_ip(&mut self) -> Result<Vec<std::net::IpAddr>, P2pNetworkServiceError> {
        P2pServiceWebrtcWithLibp2p::detect_local_ip(self)
    }
}

impl<T: P2pServiceWebrtcWithLibp2p> P2pConnectionService for T {
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> Option<P2pConnectionOutgoingInitOpts> {
        P2pServiceWebrtc::random_pick(self, list)
    }

    fn outgoing_init(&mut self, opts: P2pConnectionOutgoingInitOpts) {
        match opts {
            P2pConnectionOutgoingInitOpts::WebRTC { peer_id, .. } => {
                P2pServiceWebrtc::outgoing_init(self, peer_id);
            }
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pConnectionOutgoingInitOpts::LibP2P(_) => {}
            #[cfg(feature = "p2p-libp2p")]
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
                self.mio()
                    .send_cmd(crate::MioCmd::Connect(std::net::SocketAddr::new(
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
}

impl<T: P2pServiceWebrtcWithLibp2p> P2pDisconnectionService for T {
    fn disconnect(&mut self, peer_id: PeerId) {
        // By removing the peer, `cmd_sender` gets dropped which will
        // cause `peer_loop` to end.
        if self.peers().remove(&peer_id).is_none() {
            openmina_core::error!(openmina_core::log::system_time(); "`disconnect` shouldn't be used for libp2p peers");
        }
    }
}

impl<T: P2pServiceWebrtcWithLibp2p> P2pChannelsService for T {
    fn channel_open(&mut self, peer_id: PeerId, id: ChannelId) {
        if self.peers().contains_key(&peer_id) {
            P2pServiceWebrtc::channel_open(self, peer_id, id)
        } else if !matches!(id, ChannelId::Rpc) {
            // skip sending event for rpc libp2p channel as the ready
            // action is dispatched in the `network` module after the
            // relevant handshake is done.
            // TODO: do the same for other channels/streams also.
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
        }
    }
}

#[cfg(feature = "p2p-libp2p")]
impl<T> P2pMioService for T
where
    T: P2pServiceWebrtcWithLibp2p,
{
    #[cfg(feature = "p2p-libp2p")]
    fn start_mio(&mut self) {
        let event_sender = self.event_sender().clone();
        self.mio().run(move |mio_event| {
            event_sender
                .send(P2pEvent::MioEvent(mio_event).into())
                .unwrap_or_default()
        });
    }

    #[cfg(feature = "p2p-libp2p")]
    fn send_mio_cmd(&mut self, cmd: crate::MioCmd) {
        self.mio().send_cmd(cmd)
    }
}

impl P2pServiceCtx {
    pub fn mocked(sec_key: SecretKey) -> Self {
        use openmina_core::channels::mpsc;
        Self {
            sec_key: sec_key.clone(),
            #[cfg(feature = "p2p-libp2p")]
            mio: super::mio::MioService::mocked(sec_key.try_into().expect("valid keypair")),
            webrtc: super::webrtc::P2pServiceCtx {
                cmd_sender: mpsc::unbounded_channel().0,
                peers: Default::default(),
            },
        }
    }
}

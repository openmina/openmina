use std::collections::BTreeMap;

use ::webrtc::{
    api::APIBuilder,
    data_channel::data_channel_init::RTCDataChannelInit,
    ice_transport::{
        ice_credential_type::RTCIceCredentialType, ice_gatherer_state::RTCIceGathererState,
        ice_gathering_state::RTCIceGatheringState, ice_server::RTCIceServer,
    },
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        policy::ice_transport_policy::RTCIceTransportPolicy,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
};
use tokio::sync::{mpsc, oneshot};

use crate::{
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionService},
    disconnection::P2pDisconnectionService,
    webrtc, P2pConnectionEvent, P2pEvent, PeerId,
};

pub enum Cmd {
    PeerAdd(PeerAddArgs),
}

pub enum PeerCmd {
    PeerHttpOfferSend(String, webrtc::Offer),
    AnswerSet(webrtc::Answer),
}

pub struct P2pServiceCtx {
    pub cmd_sender: mpsc::UnboundedSender<Cmd>,
    pub peers: BTreeMap<PeerId, PeerState>,
}

pub struct PeerAddArgs {
    peer_id: PeerId,
    kind: PeerConnectionKind,
    event_sender: mpsc::UnboundedSender<P2pEvent>,
    cmd_receiver: mpsc::UnboundedReceiver<PeerCmd>,
}

pub enum PeerConnectionKind {
    Outgoing,
    Incoming(webrtc::Offer),
}

pub struct PeerState {
    cmd_sender: mpsc::UnboundedSender<PeerCmd>,
}

async fn wait_for_ice_gathering_complete(pc: &RTCPeerConnection) {
    if !matches!(pc.ice_gathering_state(), RTCIceGatheringState::Complete) {
        let (tx, rx) = oneshot::channel::<()>();
        let mut tx = Some(tx);
        pc.on_ice_gathering_state_change(Box::new(move |state| {
            if matches!(state, RTCIceGathererState::Complete) {
                if let Some(tx) = tx.take() {
                    let _ = tx.send(());
                }
            }
            Box::pin(std::future::ready(()))
        }));
        // TODO(binier): timeout
        let _ = rx.await;
    }
}

#[derive(thiserror::Error, derive_more::From, Debug)]
enum Error {
    #[error("{0}")]
    RTCError(::webrtc::Error),
    #[error("signal serialization failed: {0}")]
    SignalSerializeError(serde_json::Error),
    #[error("http request failed: {0}")]
    HyperError(hyper::Error),
    #[error("http request failed: {0}")]
    HttpError(hyper::http::Error),
    #[from(ignore)]
    #[error("channel closed")]
    ChannelClosed,
}

async fn peer_start(args: PeerAddArgs) {
    let PeerAddArgs {
        peer_id,
        kind,
        event_sender,
        mut cmd_receiver,
    } = args;
    let is_outgoing = matches!(kind, PeerConnectionKind::Outgoing);

    let webrtc = APIBuilder::new().build();

    let config = RTCConfiguration {
        ice_servers: vec![
            RTCIceServer {
                urls: vec!["stun:138.201.74.177:3478".to_owned()],
                username: "openmina".to_owned(),
                credential: "webrtc".to_owned(),
                credential_type: RTCIceCredentialType::Password,
            },
            RTCIceServer {
                urls: vec!["stun:65.109.110.75:3478".to_owned()],
                username: "openmina".to_owned(),
                credential: "webrtc".to_owned(),
                credential_type: RTCIceCredentialType::Password,
            },
            RTCIceServer {
                urls: vec!["turn:138.201.74.177:3478".to_owned()],
                username: "openmina".to_owned(),
                credential: "webrtc".to_owned(),
                credential_type: RTCIceCredentialType::Password,
            },
            RTCIceServer {
                urls: vec!["turn:65.109.110.75:3478".to_owned()],
                username: "openmina".to_owned(),
                credential: "webrtc".to_owned(),
                credential_type: RTCIceCredentialType::Password,
            },
        ],
        ice_transport_policy: RTCIceTransportPolicy::All,
        // TODO(binier): certificate
        ..Default::default()
    };
    let fut = async {
        let pc = webrtc.new_peer_connection(config).await?;
        let main_channel = pc
            .create_data_channel(
                "",
                Some(RTCDataChannelInit {
                    ordered: Some(true),
                    max_packet_life_time: None,
                    max_retransmits: None,
                    negotiated: Some(0),
                    ..Default::default()
                }),
            )
            .await?;

        let offer = match kind {
            PeerConnectionKind::Incoming(offer) => RTCSessionDescription::offer(offer.sdp)?,
            PeerConnectionKind::Outgoing => pc.create_offer(None).await?,
        };

        if is_outgoing {
            pc.set_local_description(offer).await?;
            wait_for_ice_gathering_complete(&pc).await;
        } else {
            pc.set_remote_description(offer).await?;
        }

        Result::<_, Error>::Ok((pc, main_channel))
    };

    let (pc, main_channel) = match fut.await {
        Ok(v) => v,
        Err(err) => {
            let _ = event_sender
                .send(P2pConnectionEvent::OfferSdpReady(peer_id, Err(err.to_string())).into());
            return;
        }
    };

    let answer = if is_outgoing {
        let answer_fut = async {
            let sdp = pc.local_description().await.unwrap().sdp;
            event_sender
                .send(P2pConnectionEvent::OfferSdpReady(peer_id, Ok(sdp)).into())
                .or(Err(Error::ChannelClosed))?;
            match cmd_receiver.recv().await.ok_or(Error::ChannelClosed)? {
                PeerCmd::PeerHttpOfferSend(url, offer) => {
                    let event_sender = event_sender.clone();
                    let client = hyper::Client::new();
                    let req =
                        hyper::Request::post(url).body(serde_json::to_string(&offer)?.into())?;
                    let body = client.request(req).await?.into_body();
                    let bytes = hyper::body::to_bytes(body).await?;
                    let answer = serde_json::from_slice(bytes.as_ref())?;
                    event_sender
                        .send(P2pConnectionEvent::AnswerReceived(peer_id, answer).into())
                        .or(Err(Error::ChannelClosed))?;

                    if let PeerCmd::AnswerSet(v) =
                        cmd_receiver.recv().await.ok_or(Error::ChannelClosed)?
                    {
                        return Ok(v);
                    }
                }
                PeerCmd::AnswerSet(v) => return Ok(v),
            }
            Err(Error::ChannelClosed)
        };
        answer_fut
            .await
            .and_then(|v| Ok(RTCSessionDescription::answer(v.sdp)?))
    } else {
        pc.create_answer(None).await.map_err(|e| Error::from(e))
    };
    let Ok(answer) = answer else {
        let _ = pc.close().await;
        return;
    };

    if is_outgoing {
        if let Err(err) = pc.set_remote_description(answer).await {
            let err = Error::from(err).to_string();
            let _ = pc.close().await;
            let _ = event_sender.send(P2pConnectionEvent::Finalized(peer_id, Err(err)).into());
        }
    } else {
        let fut = async {
            pc.set_local_description(answer).await?;
            wait_for_ice_gathering_complete(&pc).await;
            Ok(pc.local_description().await.unwrap().sdp)
        };
        let res = fut.await.map_err(|err: Error| err.to_string());
        let is_err = res.is_err();
        let is_err = is_err
            || event_sender
                .send(P2pConnectionEvent::AnswerSdpReady(peer_id, res).into())
                .is_err();
        if is_err {
            let _ = pc.close().await;
            return;
        }
    }

    let (connected_tx, connected) = oneshot::channel();
    if matches!(pc.connection_state(), RTCPeerConnectionState::Connected) {
        connected_tx.send(Ok(())).unwrap();
    } else {
        let mut connected_tx = Some(connected_tx);
        let event_sender = event_sender.clone();
        pc.on_peer_connection_state_change(Box::new(move |state| {
            match state {
                RTCPeerConnectionState::Connected => {
                    if let Some(connected_tx) = connected_tx.take() {
                        let _ = connected_tx.send(Ok(()));
                    }
                }
                RTCPeerConnectionState::Disconnected | RTCPeerConnectionState::Closed => {
                    if let Some(connected_tx) = connected_tx.take() {
                        let _ = connected_tx.send(Err("disconnected"));
                    } else {
                        let _ = event_sender.send(P2pConnectionEvent::Closed(peer_id).into());
                    }
                }
                _ => {}
            }
            Box::pin(std::future::ready(()))
        }));
    }
    match connected
        .await
        .map_err(|_| Error::ChannelClosed.to_string())
        .and_then(|res| res.map_err(|v| v.to_string()))
    {
        Ok(_) => {}
        Err(err) => {
            let _ = event_sender.send(P2pConnectionEvent::Finalized(peer_id, Err(err)).into());
        }
    }
    let _ = main_channel.close().await;

    let _ = event_sender.send(P2pConnectionEvent::Finalized(peer_id, Ok(())).into());

    peer_loop(event_sender, cmd_receiver, pc).await
}

// TODO(binier): remove unwraps
async fn peer_loop(
    event_sender: mpsc::UnboundedSender<P2pEvent>,
    cmd_receiver: mpsc::UnboundedReceiver<PeerCmd>,
    pc: RTCPeerConnection,
) {
    let _ = (event_sender, cmd_receiver, pc);
    // while let Some(cmd) = ctx.cmd_receiver.recv().await {
    // }
}

pub trait P2pServiceWebrtcRs: redux::Service {
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts;

    fn event_sender(&mut self) -> &mut mpsc::UnboundedSender<P2pEvent>;

    fn cmd_sender(&mut self) -> &mut mpsc::UnboundedSender<Cmd>;

    fn peers(&mut self) -> &mut BTreeMap<PeerId, PeerState>;

    fn init() -> P2pServiceCtx {
        let (cmd_sender, mut cmd_receiver) = mpsc::unbounded_channel();

        tokio::task::spawn_blocking(move || {
            let local = tokio::task::LocalSet::new();
            let main_fut = local.run_until(async {
                while let Some(cmd) = cmd_receiver.recv().await {
                    match cmd {
                        Cmd::PeerAdd(args) => {
                            tokio::task::spawn_local(peer_start(args));
                        }
                    }
                }
            });
            tokio::runtime::Handle::current().block_on(main_fut);
        });

        P2pServiceCtx {
            cmd_sender,
            peers: Default::default(),
        }
    }
}

impl<T: P2pServiceWebrtcRs> P2pConnectionService for T {
    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        P2pServiceWebrtcRs::random_pick(self, list)
    }

    fn outgoing_init(&mut self, peer_id: PeerId) {
        let (peer_cmd_sender, peer_cmd_receiver) = mpsc::unbounded_channel();

        self.peers().insert(
            peer_id,
            PeerState {
                cmd_sender: peer_cmd_sender,
            },
        );
        let event_sender = self.event_sender().clone();
        let _ = self.cmd_sender().send(Cmd::PeerAdd(PeerAddArgs {
            peer_id,
            kind: PeerConnectionKind::Outgoing,
            event_sender,
            cmd_receiver: peer_cmd_receiver,
        }));
    }

    fn incoming_init(&mut self, peer_id: PeerId, offer: webrtc::Offer) {
        let (peer_cmd_sender, peer_cmd_receiver) = mpsc::unbounded_channel();

        self.peers().insert(
            peer_id,
            PeerState {
                cmd_sender: peer_cmd_sender,
            },
        );
        let event_sender = self.event_sender().clone();
        let _ = self.cmd_sender().send(Cmd::PeerAdd(PeerAddArgs {
            peer_id,
            kind: PeerConnectionKind::Incoming(offer),
            event_sender,
            cmd_receiver: peer_cmd_receiver,
        }));
    }

    fn set_answer(&mut self, peer_id: PeerId, answer: webrtc::Answer) {
        if let Some(peer) = self.peers().get(&peer_id) {
            let _ = peer.cmd_sender.send(PeerCmd::AnswerSet(answer));
        }
    }

    fn http_signaling_request(&mut self, url: String, offer: webrtc::Offer) {
        if let Some(peer) = self.peers().get(&offer.target_peer_id) {
            let _ = peer.cmd_sender.send(PeerCmd::PeerHttpOfferSend(url, offer));
        }
    }
}

impl<T: P2pServiceWebrtcRs> P2pDisconnectionService for T {
    fn disconnect(&mut self, peer_id: PeerId) {
        // By removing the peer, `cmd_sender` gets dropped which will
        // cause `peer_loop` to end.
        self.peers().remove(&peer_id);
    }
}

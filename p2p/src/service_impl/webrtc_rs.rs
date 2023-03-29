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

async fn wait_for_ice_gathering_complete(pc: &RTCPeerConnection) -> Result<(), ()> {
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
        rx.await.unwrap();
        Ok(())
    } else {
        Ok(())
    }
}

// TODO(binier): remove unwraps
async fn peer_loop(args: PeerAddArgs) {
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
    let pc = webrtc.new_peer_connection(config).await.unwrap();
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
        .await
        .unwrap();

    let offer = match kind {
        PeerConnectionKind::Incoming(offer) => RTCSessionDescription::offer(offer.sdp).unwrap(),
        PeerConnectionKind::Outgoing => pc.create_offer(None).await.unwrap(),
    };

    if is_outgoing {
        pc.set_local_description(offer).await.unwrap();
        wait_for_ice_gathering_complete(&pc).await.unwrap();
    } else {
        pc.set_remote_description(offer).await.unwrap();
    }

    let answer = if is_outgoing {
        let sdp = pc.local_description().await.unwrap().sdp;
        event_sender
            .send(P2pConnectionEvent::OfferSdpReady(peer_id, sdp).into())
            .unwrap();

        let answer_fut = async {
            match cmd_receiver.recv().await? {
                PeerCmd::PeerHttpOfferSend(url, offer) => {
                    let event_sender = event_sender.clone();
                    let client = hyper::Client::new();
                    let req = hyper::Request::post(url)
                        .body(serde_json::to_string(&offer).unwrap().into())
                        .unwrap();
                    let body = client.request(req).await.unwrap().into_body();
                    let bytes = hyper::body::to_bytes(body).await.unwrap();
                    // TODO(binier): decoding should be inside state machine.
                    String::from_utf8(bytes.clone().to_vec()).unwrap();
                    let answer = serde_json::from_slice(bytes.as_ref()).unwrap();
                    let _ = event_sender
                        .send(P2pConnectionEvent::AnswerReceived(peer_id, answer).into());

                    if let PeerCmd::AnswerSet(v) = cmd_receiver.recv().await? {
                        return Some(v);
                    }
                }
                PeerCmd::AnswerSet(v) => return Some(v),
            }
            None
        };
        match answer_fut.await {
            Some(v) => RTCSessionDescription::answer(v.sdp).unwrap(),
            None => {
                let _ = pc.close().await;
                return;
            }
        }
    } else {
        pc.create_answer(None).await.unwrap()
    };

    if is_outgoing {
        pc.set_remote_description(answer).await.unwrap();
    } else {
        pc.set_local_description(answer).await.unwrap();
        wait_for_ice_gathering_complete(&pc).await.unwrap();
        let sdp = pc.local_description().await.unwrap().sdp;
        event_sender
            .send(P2pConnectionEvent::AnswerSdpReady(peer_id, sdp).into())
            .unwrap();
    }

    let (connected_tx, connected) = oneshot::channel::<()>();
    if matches!(pc.connection_state(), RTCPeerConnectionState::Connected) {
        connected_tx.send(()).unwrap();
    } else {
        let mut connected_tx = Some(connected_tx);
        pc.on_peer_connection_state_change(Box::new(move |state| {
            if matches!(state, RTCPeerConnectionState::Connected) {
                if let Some(connected_tx) = connected_tx.take() {
                    let _ = connected_tx.send(());
                }
            }
            Box::pin(std::future::ready(()))
        }));
    }
    connected.await.unwrap();
    main_channel.close().await.unwrap();

    event_sender
        .send(P2pConnectionEvent::Opened(peer_id).into())
        .unwrap();
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
                            tokio::task::spawn_local(peer_loop(args));
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

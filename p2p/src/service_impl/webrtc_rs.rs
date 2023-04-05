use std::{collections::BTreeMap, sync::Arc};

use ::webrtc::{
    api::APIBuilder,
    data_channel::{data_channel_init::RTCDataChannelInit, RTCDataChannel},
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
    channel::{ChannelId, ChannelMsg, MsgId, P2pChannelService},
    connection::{outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionService},
    disconnection::P2pDisconnectionService,
    webrtc, P2pChannelEvent, P2pConnectionEvent, P2pEvent, PeerId,
};

pub enum Cmd {
    PeerAdd(PeerAddArgs),
}

pub enum PeerCmd {
    PeerHttpOfferSend(String, webrtc::Offer),
    AnswerSet(webrtc::Answer),
    ChannelOpen(ChannelId),
    ChannelSend(MsgId, ChannelMsg),

    // Internally called.
    ChannelOpened(ChannelId, Result<Arc<RTCDataChannel>, ::webrtc::Error>),
    ChannelClosed(ChannelId),
}

pub struct P2pServiceCtx {
    pub cmd_sender: mpsc::UnboundedSender<Cmd>,
    pub peers: BTreeMap<PeerId, PeerState>,
}

pub struct PeerAddArgs {
    peer_id: PeerId,
    kind: PeerConnectionKind,
    event_sender: mpsc::UnboundedSender<P2pEvent>,
    cmd_sender: mpsc::UnboundedSender<PeerCmd>,
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
    #[error("unexpected cmd received")]
    UnexpectedCmd,
    #[from(ignore)]
    #[error("channel closed")]
    ChannelClosed,
}

async fn peer_start(args: PeerAddArgs) {
    let PeerAddArgs {
        peer_id,
        kind,
        event_sender,
        cmd_sender,
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
                _cmd => {
                    return Err(Error::UnexpectedCmd);
                }
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

    peer_loop(peer_id, event_sender, cmd_sender, cmd_receiver, pc).await
}

struct Channel {
    id: ChannelId,
    chan: Arc<RTCDataChannel>,
}

struct MsgBuffer {
    buf: Vec<u8>,
}

impl MsgBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    fn encode(&mut self, msg: &ChannelMsg) -> Result<Vec<u8>, std::io::Error> {
        msg.encode(&mut self.buf)?;
        let encoded = self.buf.clone();
        self.buf.clear();
        Ok(encoded)
    }
}

struct Channels {
    list: Vec<Channel>,
}

impl Channels {
    fn new() -> Self {
        Self {
            list: Vec::with_capacity(32),
        }
    }

    fn get(&self, id: ChannelId) -> Option<&Arc<RTCDataChannel>> {
        self.list.iter().find(|c| c.id == id).map(|c| &c.chan)
    }

    fn add(&mut self, id: ChannelId, chan: Arc<RTCDataChannel>) {
        self.list.push(Channel { id, chan });
    }

    fn remove(&mut self, id: ChannelId) -> Option<Arc<RTCDataChannel>> {
        let index = self.list.iter().position(|c| c.id == id)?;
        Some(self.list.remove(index).chan)
    }
}

// TODO(binier): remove unwraps
async fn peer_loop(
    peer_id: PeerId,
    event_sender: mpsc::UnboundedSender<P2pEvent>,
    cmd_sender: mpsc::UnboundedSender<PeerCmd>,
    mut cmd_receiver: mpsc::UnboundedReceiver<PeerCmd>,
    pc: RTCPeerConnection,
) {
    let pc = Arc::new(pc);
    // TODO(binier): maybe use small_vec (stack allocated) or something like that.
    let mut channels = Channels::new();
    let mut msg_buf = MsgBuffer::new(64 * 1024);

    while let Some(cmd) = cmd_receiver.recv().await {
        match cmd {
            PeerCmd::PeerHttpOfferSend(..) | PeerCmd::AnswerSet(_) => {
                // TODO(binier): log unexpected peer cmd.
            }
            PeerCmd::ChannelOpen(id) => {
                let pc = pc.clone();
                let cmd_sender = cmd_sender.clone();
                tokio::task::spawn_local(async move {
                    let cmd_sender_clone = cmd_sender.clone();
                    let result = async move {
                        let chan = pc
                            .create_data_channel(
                                id.name(),
                                Some(RTCDataChannelInit {
                                    ordered: Some(true),
                                    max_packet_life_time: None,
                                    max_retransmits: None,
                                    negotiated: Some(id.to_u16()),
                                    ..Default::default()
                                }),
                            )
                            .await?;

                        let (done_tx, mut done_rx) =
                            mpsc::channel::<Result<(), ::webrtc::Error>>(1);

                        let done_tx_clone = done_tx.clone();
                        chan.on_open(Box::new(move || {
                            let _ = done_tx_clone.try_send(Ok(()));
                            Box::pin(std::future::ready(()))
                        }));

                        let done_tx_clone = done_tx.clone();
                        let cmd_sender = cmd_sender_clone.clone();
                        chan.on_error(Box::new(move |err| {
                            if let Err(_) = done_tx_clone.try_send(Err(err)) {
                                let _ = cmd_sender.send(PeerCmd::ChannelClosed(id));
                            }
                            Box::pin(std::future::ready(()))
                        }));

                        let done_tx_clone = done_tx.clone();
                        let cmd_sender = cmd_sender_clone.clone();
                        chan.on_close(Box::new(move || {
                            if let Err(_) =
                                done_tx_clone.try_send(Err(::webrtc::Error::ErrDataChannelNotOpen))
                            {
                                let _ = cmd_sender.send(PeerCmd::ChannelClosed(id));
                            }
                            Box::pin(std::future::ready(()))
                        }));

                        done_rx
                            .recv()
                            .await
                            .ok_or(::webrtc::Error::ErrDataChannelNotOpen)??;

                        Ok(chan)
                    };

                    let _ = cmd_sender.send(PeerCmd::ChannelOpened(id, result.await));
                });
            }
            PeerCmd::ChannelSend(msg_id, msg) => {
                let id = msg.channel_id();
                let chan = match channels.get(id) {
                    Some(v) => v.clone(),
                    None => {
                        let err = "ChannelNotOpen".to_owned();
                        let _ = event_sender
                            .send(P2pChannelEvent::Sent(peer_id, id, msg_id, Err(err)).into());
                        continue;
                    }
                };

                let encoded = msg_buf.encode(&msg);

                let event_sender_clone = event_sender.clone();
                tokio::task::spawn_local(async move {
                    let result = async move {
                        match encoded {
                            Ok(encoded) => {
                                let encoded_len = encoded.len();
                                let n = chan
                                    .send(&encoded.into())
                                    .await
                                    .map_err(|e| e.to_string())?;
                                if n != encoded_len {
                                    return Err("NotAllBytesWritten".to_owned());
                                }
                                Ok(())
                            }
                            Err(err) => Err(err.to_string()),
                        }
                    };

                    let _ = event_sender_clone
                        .send(P2pChannelEvent::Sent(peer_id, id, msg_id, result.await).into());
                });
            }
            PeerCmd::ChannelOpened(chan_id, result) => {
                let res = match result {
                    Ok(chan) => {
                        channels.add(chan_id, chan);
                        Ok(())
                    }
                    Err(err) => Err(err.to_string()),
                };

                if let Some(chan) = channels.get(chan_id) {
                    let event_sender = event_sender.clone();
                    chan.on_message(Box::new(move |msg| {
                        let msg = ChannelMsg::decode(&mut msg.data.as_ref(), chan_id)
                            .map_err(|err| err.to_string());
                        let _ = event_sender.send(P2pChannelEvent::Received(peer_id, msg).into());
                        Box::pin(std::future::ready(()))
                    }));
                }

                let _ = event_sender.send(P2pChannelEvent::Opened(peer_id, chan_id, res).into());
            }
            PeerCmd::ChannelClosed(id) => {
                channels.remove(id);
                let _ = event_sender.send(P2pChannelEvent::Closed(peer_id, id).into());
            }
        }
    }
    let _ = pc.close().await;
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
                cmd_sender: peer_cmd_sender.clone(),
            },
        );
        let event_sender = self.event_sender().clone();
        let _ = self.cmd_sender().send(Cmd::PeerAdd(PeerAddArgs {
            peer_id,
            kind: PeerConnectionKind::Outgoing,
            event_sender,
            cmd_sender: peer_cmd_sender,
            cmd_receiver: peer_cmd_receiver,
        }));
    }

    fn incoming_init(&mut self, peer_id: PeerId, offer: webrtc::Offer) {
        let (peer_cmd_sender, peer_cmd_receiver) = mpsc::unbounded_channel();

        self.peers().insert(
            peer_id,
            PeerState {
                cmd_sender: peer_cmd_sender.clone(),
            },
        );
        let event_sender = self.event_sender().clone();
        let _ = self.cmd_sender().send(Cmd::PeerAdd(PeerAddArgs {
            peer_id,
            kind: PeerConnectionKind::Incoming(offer),
            event_sender,
            cmd_sender: peer_cmd_sender,
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

impl<T: P2pServiceWebrtcRs> P2pChannelService for T {
    fn channel_open(&mut self, peer_id: PeerId, id: ChannelId) {
        if let Some(peer) = self.peers().get(&peer_id) {
            let _ = peer.cmd_sender.send(PeerCmd::ChannelOpen(id));
        }
    }

    fn channel_send(&mut self, peer_id: PeerId, msg_id: MsgId, msg: ChannelMsg) {
        if let Some(peer) = self.peers().get(&peer_id) {
            let _ = peer.cmd_sender.send(PeerCmd::ChannelSend(msg_id, msg));
        }
    }
}

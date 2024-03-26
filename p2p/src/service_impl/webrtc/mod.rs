#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::{collections::BTreeMap, time::Duration};

use serde::Serialize;

#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use openmina_core::channels::{mpsc, oneshot};

use crate::{
    channels::{ChannelId, ChannelMsg, MsgId},
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    identity::SecretKey,
    webrtc, P2pChannelEvent, P2pConnectionEvent, P2pEvent, PeerId,
};

#[cfg(not(target_arch = "wasm32"))]
use self::native::{
    webrtc_signal_send, RTCChannel, RTCConnection, RTCConnectionState, RTCSignalingError,
};
#[cfg(target_arch = "wasm32")]
use self::web::{
    webrtc_signal_send, RTCChannel, RTCConnection, RTCConnectionState, RTCSignalingError,
};

use super::TaskSpawner;

/// 16KB.
const CHUNK_SIZE: usize = 16 * 1024;

pub enum Cmd {
    PeerAdd(PeerAddArgs),
}

pub enum PeerCmd {
    PeerHttpOfferSend(String, webrtc::Offer),
    AnswerSet(webrtc::Answer),
    ChannelOpen(ChannelId),
    ChannelSend(MsgId, ChannelMsg),
}

enum PeerCmdInternal {
    ChannelOpened(ChannelId, Result<RTCChannel, Error>),
    ChannelClosed(ChannelId),
}

enum PeerCmdAll {
    External(PeerCmd),
    Internal(PeerCmdInternal),
}

pub struct P2pServiceCtx {
    pub cmd_sender: mpsc::UnboundedSender<Cmd>,
    pub peers: BTreeMap<PeerId, PeerState>,
}

pub struct PeerAddArgs {
    peer_id: PeerId,
    kind: PeerConnectionKind,
    event_sender: Arc<dyn Fn(P2pEvent) -> Option<()> + Send + Sync + 'static>,
    cmd_receiver: mpsc::UnboundedReceiver<PeerCmd>,
}

pub enum PeerConnectionKind {
    Outgoing,
    Incoming(webrtc::Offer),
}

pub struct PeerState {
    cmd_sender: mpsc::UnboundedSender<PeerCmd>,
}

#[derive(thiserror::Error, derive_more::From, Debug)]
pub(super) enum Error {
    #[cfg(not(target_arch = "wasm32"))]
    #[error("{0}")]
    RTCError(::webrtc::Error),
    #[cfg(target_arch = "wasm32")]
    #[error("js error: {0:?}")]
    RTCJsError(String),
    #[error("signaling error: {0}")]
    SignalingError(RTCSignalingError),
    #[error("unexpected cmd received")]
    UnexpectedCmd,
    #[from(ignore)]
    #[error("channel closed")]
    ChannelClosed,
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for Error {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Error::RTCJsError(format!("{value:?}"))
    }
}

pub type OnConnectionStateChangeHdlrFn = Box<
    dyn (FnMut(RTCConnectionState) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>)
        + Send
        + Sync,
>;

pub struct RTCConfig {
    pub ice_servers: RTCConfigIceServers,
    // TODO(binier): certificate
}

#[derive(Serialize)]
pub struct RTCConfigIceServers(Vec<RTCConfigIceServer>);
#[derive(Serialize)]
pub struct RTCConfigIceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

#[derive(Serialize)]
pub struct RTCChannelConfig {
    pub label: &'static str,
    pub negotiated: Option<u16>,
}

impl Default for RTCConfigIceServers {
    fn default() -> Self {
        Self(vec![
            RTCConfigIceServer {
                urls: vec!["stun:65.109.110.75:3478".to_owned()],
                username: Some("openmina".to_owned()),
                credential: Some("webrtc".to_owned()),
            },
            RTCConfigIceServer {
                urls: vec![
                    "stun:stun.l.google.com:19302".to_owned(),
                    "stun:stun1.l.google.com:19302".to_owned(),
                    "stun:stun2.l.google.com:19302".to_owned(),
                    "stun:stun3.l.google.com:19302".to_owned(),
                    "stun:stun4.l.google.com:19302".to_owned(),
                ],
                username: None,
                credential: None,
            },
        ])
    }
}

impl std::ops::Deref for RTCConfigIceServers {
    type Target = Vec<RTCConfigIceServer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for RTCConnection {
    fn drop(&mut self) {
        if self.is_main() {
            let cloned = self.clone();
            spawn_local(async move {
                let _ = cloned.close().await;
            });
        }
    }
}

impl Drop for RTCChannel {
    fn drop(&mut self) {
        if self.is_main() {
            let cloned = self.clone();
            spawn_local(async move {
                let _ = cloned.close().await;
            });
        }
    }
}

async fn wait_for_ice_gathering_complete(pc: &RTCConnection) {
    let timeout = Duration::from_secs(3);

    #[cfg(not(target_arch = "wasm32"))]
    let timeout = tokio::time::sleep(timeout);
    #[cfg(target_arch = "wasm32")]
    let timeout = wasm_timer::Delay::new(timeout);
    tokio::select! {
        _ = timeout => {}
        _ = pc.wait_for_ice_gathering_complete() => {}
    }
}

// TODO(binier): cancel future if peer cmd sender is dropped.
async fn peer_start(args: PeerAddArgs) {
    let PeerAddArgs {
        peer_id,
        kind,
        event_sender,
        mut cmd_receiver,
    } = args;
    let is_outgoing = matches!(kind, PeerConnectionKind::Outgoing);

    let config = RTCConfig {
        ice_servers: Default::default(),
    };
    let fut = async {
        let pc = RTCConnection::create(config).await?;
        let main_channel = pc
            .channel_create(RTCChannelConfig {
                label: "",
                negotiated: Some(0),
            })
            .await?;

        let offer = match kind {
            PeerConnectionKind::Incoming(offer) => offer.try_into()?,
            PeerConnectionKind::Outgoing => pc.offer_create().await?,
        };

        if is_outgoing {
            pc.local_desc_set(offer).await?;
            wait_for_ice_gathering_complete(&pc).await;
        } else {
            pc.remote_desc_set(offer).await?;
        }

        Result::<_, Error>::Ok((pc, main_channel))
    };

    let (pc, main_channel) = match fut.await {
        Ok(v) => v,
        Err(err) => {
            event_sender(P2pConnectionEvent::OfferSdpReady(peer_id, Err(err.to_string())).into());
            return;
        }
    };

    let answer = if is_outgoing {
        let answer_fut = async {
            let sdp = pc.local_sdp().await.unwrap();
            event_sender(P2pConnectionEvent::OfferSdpReady(peer_id, Ok(sdp)).into())
                .ok_or(Error::ChannelClosed)?;
            match cmd_receiver.recv().await.ok_or(Error::ChannelClosed)? {
                PeerCmd::PeerHttpOfferSend(url, offer) => {
                    let answer = webrtc_signal_send(&url, offer).await?;
                    event_sender(P2pConnectionEvent::AnswerReceived(peer_id, answer).into())
                        .ok_or(Error::ChannelClosed)?;

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
        answer_fut.await.and_then(|v| Ok(v.try_into()?))
    } else {
        pc.answer_create().await.map_err(Error::from)
    };
    let Ok(answer) = answer else {
        return;
    };

    if is_outgoing {
        if let Err(err) = pc.remote_desc_set(answer).await {
            let err = Error::from(err).to_string();
            let _ = event_sender(P2pConnectionEvent::Finalized(peer_id, Err(err)).into());
        }
    } else {
        let fut = async {
            pc.local_desc_set(answer).await?;
            wait_for_ice_gathering_complete(&pc).await;
            Ok(pc.local_sdp().await.unwrap())
        };
        let res = fut.await.map_err(|err: Error| err.to_string());
        let is_err = res.is_err();
        let is_err = is_err
            || event_sender(P2pConnectionEvent::AnswerSdpReady(peer_id, res).into()).is_none();
        if is_err {
            return;
        }
    }

    let (connected_tx, connected) = oneshot::channel();
    if matches!(pc.connection_state(), RTCConnectionState::Connected) {
        connected_tx.send(Ok(())).unwrap();
    } else {
        let mut connected_tx = Some(connected_tx);
        let event_sender = event_sender.clone();
        pc.on_connection_state_change(Box::new(move |state| {
            match state {
                RTCConnectionState::Connected => {
                    if let Some(connected_tx) = connected_tx.take() {
                        let _ = connected_tx.send(Ok(()));
                    }
                }
                RTCConnectionState::Disconnected | RTCConnectionState::Closed => {
                    if let Some(connected_tx) = connected_tx.take() {
                        let _ = connected_tx.send(Err("disconnected"));
                    } else {
                        let _ = event_sender(P2pConnectionEvent::Closed(peer_id).into());
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
            let _ = event_sender(P2pConnectionEvent::Finalized(peer_id, Err(err)).into());
        }
    }
    let _ = main_channel.close().await;

    let _ = event_sender(P2pConnectionEvent::Finalized(peer_id, Ok(())).into());

    peer_loop(peer_id, event_sender, cmd_receiver, pc).await
}

struct Channel {
    id: ChannelId,
    chan: RTCChannel,
    msg_sender: mpsc::UnboundedSender<(MsgId, Vec<u8>)>,
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
        let len_encoded = (self.buf.len() as u32).to_be_bytes();
        let encoded = len_encoded
            .into_iter()
            .chain(self.buf.iter().cloned())
            .collect();
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

    fn get(&self, id: ChannelId) -> Option<&RTCChannel> {
        self.list.iter().find(|c| c.id == id).map(|c| &c.chan)
    }

    fn get_msg_sender(&self, id: ChannelId) -> Option<&mpsc::UnboundedSender<(MsgId, Vec<u8>)>> {
        self.list.iter().find(|c| c.id == id).map(|c| &c.msg_sender)
    }

    fn add(
        &mut self,
        id: ChannelId,
        chan: RTCChannel,
        msg_sender: mpsc::UnboundedSender<(MsgId, Vec<u8>)>,
    ) {
        self.list.push(Channel {
            id,
            chan,
            msg_sender,
        });
    }

    fn remove(&mut self, id: ChannelId) -> Option<RTCChannel> {
        let index = self.list.iter().position(|c| c.id == id)?;
        Some(self.list.remove(index).chan)
    }
}

// TODO(binier): remove unwraps
async fn peer_loop(
    peer_id: PeerId,
    event_sender: Arc<dyn Fn(P2pEvent) -> Option<()> + Send + Sync + 'static>,
    mut cmd_receiver: mpsc::UnboundedReceiver<PeerCmd>,
    pc: RTCConnection,
) {
    // TODO(binier): maybe use small_vec (stack allocated) or something like that.
    let mut channels = Channels::new();
    let mut msg_buf = MsgBuffer::new(64 * 1024);

    let (internal_cmd_sender, mut internal_cmd_receiver) =
        mpsc::unbounded_channel::<PeerCmdInternal>();

    loop {
        let cmd = tokio::select! {
            cmd = cmd_receiver.recv() => match cmd {
                None => return,
                Some(cmd) => PeerCmdAll::External(cmd),
            },
            cmd = internal_cmd_receiver.recv() => match cmd {
                None => return,
                Some(cmd) => PeerCmdAll::Internal(cmd),
            },
        };
        match cmd {
            PeerCmdAll::External(PeerCmd::PeerHttpOfferSend(..) | PeerCmd::AnswerSet(_)) => {
                // TODO(binier): log unexpected peer cmd.
            }
            PeerCmdAll::External(PeerCmd::ChannelOpen(id)) => {
                let pc = pc.clone();
                let internal_cmd_sender = internal_cmd_sender.clone();
                spawn_local(async move {
                    let internal_cmd_sender_clone = internal_cmd_sender.clone();
                    let result = async move {
                        let chan = pc
                            .channel_create(RTCChannelConfig {
                                label: id.name(),
                                negotiated: Some(id.to_u16()),
                            })
                            .await?;

                        let (done_tx, mut done_rx) = mpsc::channel::<Result<(), Error>>(1);

                        let done_tx_clone = done_tx.clone();
                        chan.on_open(move || {
                            let _ = done_tx_clone.try_send(Ok(()));
                            std::future::ready(())
                        });

                        let done_tx_clone = done_tx.clone();
                        let internal_cmd_sender = internal_cmd_sender_clone.clone();
                        chan.on_error(move |err| {
                            if done_tx_clone.try_send(Err(err.into())).is_err() {
                                let _ =
                                    internal_cmd_sender.send(PeerCmdInternal::ChannelClosed(id));
                            }
                            std::future::ready(())
                        });

                        let done_tx_clone = done_tx.clone();
                        let internal_cmd_sender = internal_cmd_sender_clone.clone();
                        chan.on_close(move || {
                            if done_tx_clone.try_send(Err(Error::ChannelClosed)).is_err() {
                                let _ =
                                    internal_cmd_sender.send(PeerCmdInternal::ChannelClosed(id));
                            }
                            std::future::ready(())
                        });

                        done_rx.recv().await.ok_or(Error::ChannelClosed)??;

                        Ok(chan)
                    };

                    let _ =
                        internal_cmd_sender.send(PeerCmdInternal::ChannelOpened(id, result.await));
                });
            }
            PeerCmdAll::External(PeerCmd::ChannelSend(msg_id, msg)) => {
                let id = msg.channel_id();
                let err = match channels.get_msg_sender(id) {
                    Some(msg_sender) => match msg_buf.encode(&msg) {
                        Ok(encoded) => match msg_sender.send((msg_id, encoded)) {
                            Ok(_) => None,
                            Err(_) => Some("ChannelMsgMpscSendFailed".to_owned()),
                        },
                        Err(err) => Some(err.to_string()),
                    },
                    None => Some("ChannelNotOpen".to_owned()),
                };
                if let Some(err) = err {
                    let _ =
                        event_sender(P2pChannelEvent::Sent(peer_id, id, msg_id, Err(err)).into());
                }
            }
            PeerCmdAll::Internal(PeerCmdInternal::ChannelOpened(chan_id, result)) => {
                let (sender_tx, mut sender_rx) = mpsc::unbounded_channel();
                let res = match result {
                    Ok(chan) => {
                        channels.add(chan_id, chan, sender_tx);
                        Ok(())
                    }
                    Err(err) => Err(err.to_string()),
                };

                if let Some(chan) = channels.get(chan_id) {
                    let chan_clone = chan.clone();
                    let event_sender_clone = event_sender.clone();
                    spawn_local(async move {
                        while let Some((msg_id, encoded)) = sender_rx.recv().await {
                            let encoded = bytes::Bytes::from(encoded);
                            let mut chunks =
                                encoded.chunks(CHUNK_SIZE).map(|b| encoded.slice_ref(b));
                            let result = loop {
                                let Some(chunk) = chunks.next() else {
                                    break Ok(());
                                };
                                if let Err(err) = chan_clone
                                    .send(&chunk)
                                    .await
                                    .map_err(|e| format!("{e:?}"))
                                    .and_then(|n| match n == chunk.len() {
                                        false => Err("NotAllBytesWritten".to_owned()),
                                        true => Ok(()),
                                    })
                                {
                                    break Err(err);
                                }
                            };

                            let _ = event_sender_clone(
                                P2pChannelEvent::Sent(peer_id, chan_id, msg_id, result).into(),
                            );
                        }
                    });

                    fn process_msg(
                        chan_id: ChannelId,
                        buf: &mut Vec<u8>,
                        len: &mut u32,
                        msg: &mut &[u8],
                    ) -> Result<Option<ChannelMsg>, String> {
                        let len = if buf.is_empty() {
                            if msg.len() < 4 {
                                return Err("WebRTCMessageTooSmall".to_owned());
                            } else {
                                *len = u32::from_be_bytes(msg[..4].try_into().unwrap());
                                *msg = &msg[4..];
                                let len = *len as usize;
                                if len > chan_id.max_msg_size() {
                                    return Err(format!(
                                        "ChannelMsgLenOverLimit; len: {}, limit: {}",
                                        len,
                                        chan_id.max_msg_size()
                                    ));
                                }
                                len
                            }
                        } else {
                            *len as usize
                        };
                        let bytes_left = len - buf.len();

                        if bytes_left > msg.len() {
                            buf.extend_from_slice(msg);
                            *msg = &[];
                            return Ok(None);
                        }

                        buf.extend_from_slice(&msg[..bytes_left]);
                        *msg = &msg[bytes_left..];
                        let msg = ChannelMsg::decode(&mut &buf[..], chan_id)
                            .map_err(|err| err.to_string())?;
                        buf.clear();
                        Ok(Some(msg))
                    }

                    let mut len = 0;
                    let mut buf = vec![];
                    let event_sender = event_sender.clone();

                    chan.on_message(move |data| {
                        let mut data = data;
                        while !data.is_empty() {
                            let res = match process_msg(chan_id, &mut buf, &mut len, &mut data) {
                                Ok(None) => continue,
                                Ok(Some(msg)) => Ok(msg),
                                Err(err) => Err(err),
                            };
                            let _ = event_sender(P2pChannelEvent::Received(peer_id, res).into());
                        }
                        std::future::ready(())
                    });
                }

                let _ = event_sender(P2pChannelEvent::Opened(peer_id, chan_id, res).into());
            }
            PeerCmdAll::Internal(PeerCmdInternal::ChannelClosed(id)) => {
                channels.remove(id);
                let _ = event_sender(P2pChannelEvent::Closed(peer_id, id).into());
            }
        }
    }
}

pub trait P2pServiceWebrtc: redux::Service {
    type Event: From<P2pEvent> + Send + Sync + 'static;

    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts;

    fn event_sender(&mut self) -> &mut mpsc::UnboundedSender<Self::Event>;

    fn cmd_sender(&mut self) -> &mut mpsc::UnboundedSender<Cmd>;

    fn peers(&mut self) -> &mut BTreeMap<PeerId, PeerState>;

    fn init<S: TaskSpawner>(secret_key: SecretKey, spawner: S) -> P2pServiceCtx {
        let (cmd_sender, mut cmd_receiver) = mpsc::unbounded_channel();

        // TODO: sing/verify SDP
        let _ = secret_key;

        spawner.spawn_main("webrtc", async move {
            while let Some(cmd) = cmd_receiver.recv().await {
                match cmd {
                    Cmd::PeerAdd(args) => {
                        spawn_local(peer_start(args));
                    }
                }
            }
        });

        P2pServiceCtx {
            cmd_sender,
            peers: Default::default(),
        }
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
        let event_sender =
            Arc::new(move |p2p_event: P2pEvent| event_sender.send(p2p_event.into()).ok());
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
        let event_sender =
            Arc::new(move |p2p_event: P2pEvent| event_sender.send(p2p_event.into()).ok());
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

    fn disconnect(&mut self, peer_id: PeerId) {
        // By removing the peer, `cmd_sender` gets dropped which will
        // cause `peer_loop` to end.
        self.peers().remove(&peer_id);
    }

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

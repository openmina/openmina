use std::future::Future;

use datachannel::{
    sdp::parse_sdp, ConnectionState, DataChannelHandler, DataChannelInit, GatheringState,
    PeerConnectionHandler, Reliability, RtcConfig, RtcDataChannel, RtcPeerConnection, SdpType,
    SessionDescription,
};
use openmina_core::channels::{oneshot, watch};
use tokio::task::spawn_local;

use crate::{
    connection::P2pConnectionResponse,
    webrtc::{Answer, Offer},
};

use super::{OnConnectionStateChangeHdlrFn, RTCChannelConfig, RTCConfig};

pub type Error = datachannel::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub type RTCConnectionState = ConnectionState;

pub type Api = ();

type MessageHandler = Box<dyn FnMut(&[u8]) + 'static + Send>;

pub fn build_api() -> Api {}

pub struct RTCConnection {
    conn: Box<RtcPeerConnection<RTCConnectionHandlers>>,
    connection_state: watch::Receiver<RTCConnectionState>,
    gathering_state: watch::Receiver<GatheringState>,
}

pub struct RTCConnectionHandlers {
    connection_state: watch::Sender<RTCConnectionState>,
    gathering_state: watch::Sender<GatheringState>,
}

pub struct RTCChannel {
    chan: Box<RtcDataChannel<RTCChannelHandlers>>,
    opened: watch::Receiver<bool>,
    errored: watch::Receiver<Option<String>>,
    message_handler: Option<oneshot::Sender<MessageHandler>>,
}

pub struct RTCChannelHandlers {
    opened: watch::Sender<bool>,
    errored: watch::Sender<Option<String>>,
    message_handler_rx: oneshot::Receiver<MessageHandler>,
    message_handler: Option<MessageHandler>,
}

#[derive(thiserror::Error, derive_more::From, Debug)]
pub enum RTCSignalingError {
    #[error("serialization failed: {0}")]
    Serialize(serde_json::Error),
    #[error("http request failed: {0}")]
    Http(reqwest::Error),
}

impl RTCConnection {
    pub async fn create(_: &Api, config: RTCConfig) -> Result<Self> {
        let (connection_state_tx, connection_state) = watch::channel(RTCConnectionState::New);
        let (gathering_state_tx, gathering_state) = watch::channel(GatheringState::New);

        let handlers = RTCConnectionHandlers {
            connection_state: connection_state_tx,
            gathering_state: gathering_state_tx,
        };
        let conn = RtcPeerConnection::new(&config.into(), handlers)?;

        Ok(Self {
            conn,
            connection_state,
            gathering_state,
        })
    }

    pub async fn channel_create(&mut self, config: RTCChannelConfig) -> Result<RTCChannel> {
        let (opened_tx, opened) = watch::channel(false);
        let (errored_tx, errored) = watch::channel(None);
        let (message_handler, message_handler_rx) = oneshot::channel();

        let handlers = RTCChannelHandlers {
            opened: opened_tx,
            errored: errored_tx,
            message_handler_rx,
            message_handler: None,
        };

        let chan = self
            .conn
            .create_data_channel_ex(config.label, handlers, &config.into())?;

        Ok(RTCChannel {
            chan,
            opened,
            errored,
            message_handler: Some(message_handler),
        })
    }

    pub async fn offer_create(&mut self) -> Result<SessionDescription> {
        self.conn.local_description().ok_or(Error::NotAvailable)
    }

    pub async fn answer_create(&mut self) -> Result<SessionDescription> {
        self.conn.local_description().ok_or(Error::NotAvailable)
    }

    pub async fn local_desc_set(&self, _desc: SessionDescription) -> Result<()> {
        // set by `offer_create`
        Ok(())
    }

    pub async fn remote_desc_set(&mut self, desc: SessionDescription) -> Result<()> {
        self.conn.set_remote_description(&desc)
    }

    pub async fn local_sdp(&self) -> Option<String> {
        self.conn.local_description().map(|v| v.sdp.to_string())
    }

    pub fn connection_state(&self) -> RTCConnectionState {
        *self.connection_state.borrow()
    }

    pub async fn wait_for_ice_gathering_complete(&mut self) {
        let _ = self
            .gathering_state
            .wait_for(|s| matches!(s, GatheringState::Complete))
            .await;
    }

    pub fn on_connection_state_change(&self, mut handler: OnConnectionStateChangeHdlrFn) {
        let mut rx = self.connection_state.clone();
        spawn_local(async move {
            while rx.changed().await.is_ok() {
                handler(*rx.borrow()).await;
            }
        });
    }
}

impl PeerConnectionHandler for RTCConnectionHandlers {
    type DCH = RTCChannelHandlers;

    fn data_channel_handler(&mut self, _info: datachannel::DataChannelInfo) -> Self::DCH {
        RTCChannelHandlers {
            opened: watch::Sender::new(false),
            errored: watch::Sender::new(None),
            message_handler_rx: oneshot::channel().1,
            message_handler: None,
        }
    }

    fn on_connection_state_change(&mut self, state: ConnectionState) {
        let _ = self.connection_state.send(state);
    }

    fn on_gathering_state_change(&mut self, state: GatheringState) {
        let _ = self.gathering_state.send(state);
    }
}

async fn wait_opened(rx: &mut watch::Receiver<bool>) -> bool {
    rx.wait_for(|opened| *opened).await.is_ok()
}

async fn wait_closed(rx: &mut watch::Receiver<bool>) -> bool {
    if !wait_opened(rx).await {
        return false;
    }

    rx.wait_for(|opened| !*opened).await.is_ok()
}

impl RTCChannel {
    pub fn on_open<Fut>(&self, f: impl FnOnce() -> Fut + Send + Sync + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut rx = self.opened.clone();
        tokio::task::spawn_local(async move {
            if wait_opened(&mut rx).await {
                f().await;
            }
        });
    }

    pub fn on_message(&mut self, f: impl FnMut(&[u8]) + 'static + Send) {
        if let Some(tx) = self.message_handler.take() {
            let _ = tx.send(Box::new(f));
        } else {
            panic!("tried to set an already set message_handler");
        }
    }

    pub fn on_error<Fut>(&self, mut f: impl FnMut(Error) -> Fut + Send + Sync + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut rx = self.errored.clone();
        tokio::task::spawn_local(async move {
            if let Ok(err) = rx.wait_for(|err| err.is_some()).await {
                let err = Error::BadString(err.as_ref().unwrap().clone());
                f(err).await;
            }
        });
    }

    pub fn on_close<Fut>(&self, mut f: impl FnMut() -> Fut + Send + Sync + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut rx = self.opened.clone();
        tokio::task::spawn_local(async move {
            if wait_closed(&mut rx).await {
                f().await;
            }
        });
    }

    pub async fn send(&mut self, data: &bytes::Bytes) -> Result<usize> {
        self.chan.send(data.as_ref()).map(|_| data.len())
    }

    pub async fn close(&self) {}
}

impl DataChannelHandler for RTCChannelHandlers {
    fn on_open(&mut self) {
        let _ = self.opened.send(true);
    }

    fn on_message(&mut self, msg: &[u8]) {
        let f = if let Some(f) = self.message_handler.as_mut() {
            f
        } else if let Ok(f) = self.message_handler_rx.try_recv() {
            self.message_handler.insert(f)
        } else {
            return;
        };
        f(msg)
    }

    fn on_error(&mut self, err: &str) {
        let _ = self.errored.send(Some(err.to_owned()));
    }

    fn on_closed(&mut self) {
        let _ = self.opened.send(false);
    }
}

pub async fn webrtc_signal_send(
    url: &str,
    offer: Offer,
) -> std::result::Result<P2pConnectionResponse, RTCSignalingError> {
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .body(serde_json::to_string(&offer)?)
        .send()
        .await?
        .json()
        .await?;
    Ok(res)
}

impl From<RTCConfig> for RtcConfig {
    fn from(value: RTCConfig) -> Self {
        let ice_servers = value
            .ice_servers
            .0
            .into_iter()
            .flat_map(|s| {
                let needs_auth = s.credential.is_some();
                s.urls
                    .into_iter()
                    // passing credentials not supported.
                    .filter(move |_| !needs_auth)
            })
            .collect::<Vec<_>>();
        RtcConfig::new(&ice_servers)
    }
}

impl From<RTCChannelConfig> for DataChannelInit {
    fn from(value: RTCChannelConfig) -> Self {
        let reliability = Reliability::default();
        assert!(!reliability.unordered);
        assert!(!reliability.unreliable);
        assert_eq!(reliability.max_packet_life_time, 0);
        assert_eq!(reliability.max_retransmits, 0);

        let config = DataChannelInit::default().reliability(reliability);
        if let Some(stream_id) = value.negotiated {
            config.negotiated().manual_stream().stream(stream_id)
        } else {
            config
        }
    }
}

impl TryFrom<Offer> for SessionDescription {
    type Error = Error;

    fn try_from(value: Offer) -> Result<Self> {
        Ok(Self {
            sdp_type: SdpType::Offer,
            sdp: parse_sdp(&value.sdp, true).map_err(|err| Error::BadString(err.to_string()))?,
        })
    }
}

impl TryFrom<Answer> for SessionDescription {
    type Error = Error;

    fn try_from(value: Answer) -> Result<Self> {
        Ok(Self {
            sdp_type: SdpType::Answer,
            sdp: parse_sdp(&value.sdp, true).map_err(|err| Error::BadString(err.to_string()))?,
        })
    }
}

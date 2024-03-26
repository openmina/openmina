use std::future::Future;
use std::sync::Arc;

use webrtc::{
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

use crate::{
    connection::P2pConnectionResponse,
    webrtc::{Answer, Offer},
};

use super::{OnConnectionStateChangeHdlrFn, RTCChannelConfig, RTCConfig};

pub type Result<T> = std::result::Result<T, webrtc::Error>;

pub type RTCConnectionState = RTCPeerConnectionState;

pub struct RTCConnection(Arc<RTCPeerConnection>, bool);

pub struct RTCChannel(Arc<RTCDataChannel>, bool);

#[derive(thiserror::Error, derive_more::From, Debug)]
pub enum RTCSignalingError {
    #[error("serialization failed: {0}")]
    Serialize(serde_json::Error),
    #[error("http request failed: {0}")]
    Hyper(hyper::Error),
    #[error("http request failed: {0}")]
    Http(hyper::http::Error),
}

impl RTCConnection {
    pub async fn create(config: RTCConfig) -> Result<Self> {
        let webrtc = APIBuilder::new().build();
        webrtc
            .new_peer_connection(config.into())
            .await
            .map(|v| Self(v.into(), true))
    }

    pub fn is_main(&self) -> bool {
        self.1
    }

    pub async fn channel_create(&self, config: RTCChannelConfig) -> Result<RTCChannel> {
        self.0
            .create_data_channel(
                config.label,
                Some(RTCDataChannelInit {
                    ordered: Some(true),
                    max_packet_life_time: None,
                    max_retransmits: None,
                    negotiated: config.negotiated,
                    ..Default::default()
                }),
            )
            .await
            .map(|chan| RTCChannel(chan, true))
    }

    pub async fn offer_create(&self) -> Result<RTCSessionDescription> {
        self.0.create_offer(None).await
    }

    pub async fn answer_create(&self) -> Result<RTCSessionDescription> {
        self.0.create_answer(None).await
    }

    pub async fn local_desc_set(&self, desc: RTCSessionDescription) -> Result<()> {
        self.0.set_local_description(desc).await
    }

    pub async fn remote_desc_set(&self, desc: RTCSessionDescription) -> Result<()> {
        self.0.set_remote_description(desc).await
    }

    pub async fn local_sdp(&self) -> Option<String> {
        self.0.local_description().await.map(|v| v.sdp)
    }

    pub fn connection_state(&self) -> RTCConnectionState {
        self.0.connection_state()
    }

    pub async fn wait_for_ice_gathering_complete(&self) {
        if !matches!(self.0.ice_gathering_state(), RTCIceGatheringState::Complete) {
            let (tx, rx) = tokio::sync::oneshot::channel::<()>();
            let mut tx = Some(tx);
            self.0.on_ice_gathering_state_change(Box::new(move |state| {
                if matches!(state, RTCIceGathererState::Complete) {
                    if let Some(tx) = tx.take() {
                        let _ = tx.send(());
                    }
                }
                Box::pin(std::future::ready(()))
            }));
            let _ = rx.await;
        }
    }

    pub fn on_connection_state_change(&self, handler: OnConnectionStateChangeHdlrFn) {
        self.0.on_peer_connection_state_change(handler)
    }

    pub async fn close(self) {
        let _ = self.0.close().await;
    }
}

impl RTCChannel {
    pub fn is_main(&self) -> bool {
        self.1
    }

    pub fn on_open<Fut>(&self, f: impl FnOnce() -> Fut + Send + Sync + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.0.on_open(Box::new(move || Box::pin(f())))
    }

    pub fn on_message<Fut>(&self, mut f: impl FnMut(&[u8]) -> Fut + Send + Sync + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.0
            .on_message(Box::new(move |msg| Box::pin(f(msg.data.as_ref()))));
    }

    pub fn on_error<Fut>(&self, mut f: impl FnMut(webrtc::Error) -> Fut + Send + Sync + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.0.on_error(Box::new(move |err| Box::pin(f(err))))
    }

    pub fn on_close<Fut>(&self, mut f: impl FnMut() -> Fut + Send + Sync + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.0.on_close(Box::new(move || Box::pin(f())))
    }

    pub async fn send(&self, data: &bytes::Bytes) -> Result<usize> {
        self.0.send(data).await
    }

    pub async fn close(&self) {
        let _ = self.0.close().await;
    }
}

pub async fn webrtc_signal_send(
    url: &str,
    offer: Offer,
) -> std::result::Result<P2pConnectionResponse, RTCSignalingError> {
    let client = hyper::Client::new();
    let req = hyper::Request::post(url).body(serde_json::to_string(&offer)?.into())?;
    let body = client.request(req).await?.into_body();
    let bytes = hyper::body::to_bytes(body).await?;
    Ok(serde_json::from_slice(bytes.as_ref())?)
}

impl Clone for RTCConnection {
    fn clone(&self) -> Self {
        Self(self.0.clone(), false)
    }
}

impl Clone for RTCChannel {
    fn clone(&self) -> Self {
        Self(self.0.clone(), false)
    }
}

impl From<RTCConfig> for RTCConfiguration {
    fn from(value: RTCConfig) -> Self {
        RTCConfiguration {
            ice_servers: value.ice_servers.0.into_iter().map(Into::into).collect(),
            ice_transport_policy: RTCIceTransportPolicy::All,
            ..Default::default()
        }
    }
}

impl From<super::RTCConfigIceServer> for RTCIceServer {
    fn from(value: super::RTCConfigIceServer) -> Self {
        let credential_type = match value.credential.is_some() {
            false => RTCIceCredentialType::Unspecified,
            true => RTCIceCredentialType::Password,
        };
        RTCIceServer {
            urls: value.urls,
            username: value.username.unwrap_or(String::new()),
            credential: value.credential.unwrap_or(String::new()),
            credential_type,
        }
    }
}

impl TryFrom<Offer> for RTCSessionDescription {
    type Error = webrtc::Error;

    fn try_from(value: Offer) -> Result<Self> {
        Self::offer(value.sdp)
    }
}

impl TryFrom<Answer> for RTCSessionDescription {
    type Error = webrtc::Error;

    fn try_from(value: Answer) -> Result<Self> {
        Self::answer(value.sdp)
    }
}

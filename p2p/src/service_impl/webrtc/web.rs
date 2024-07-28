use std::future::Future;

use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelInit, RtcIceGatheringState,
    RtcIceTransportPolicy, RtcPeerConnection, RtcPeerConnectionState, RtcSdpType,
    RtcSessionDescriptionInit,
};

use openmina_core::channels::oneshot;

use crate::{
    connection::P2pConnectionResponse,
    webrtc::{Answer, Offer},
};

use super::{OnConnectionStateChangeHdlrFn, RTCChannelConfig, RTCConfig};

pub type Result<T> = std::result::Result<T, JsValue>;

pub type RTCConnectionState = RtcPeerConnectionState;

pub struct RTCConnection(RtcPeerConnection, bool);

pub struct RTCChannel(RtcDataChannel, bool);

#[derive(thiserror::Error, derive_more::From, Debug)]
pub enum RTCSignalingError {
    #[error("serialization failed: {0}")]
    SerializeError(serde_json::Error),
    #[error("http request failed: {0}")]
    HttpError(String),
}

impl From<JsValue> for RTCSignalingError {
    fn from(value: JsValue) -> Self {
        Self::HttpError(format!("{value:?}"))
    }
}

impl RTCConnection {
    pub async fn create(config: RTCConfig) -> Result<Self> {
        RtcPeerConnection::new_with_configuration(&config.into()).map(|v| Self(v, true))
    }

    pub fn is_main(&self) -> bool {
        self.1
    }

    pub async fn channel_create(&self, config: RTCChannelConfig) -> Result<RTCChannel> {
        let chan = self
            .0
            .create_data_channel_with_data_channel_dict(&config.label, &(&config).into());
        Ok(RTCChannel(chan, true))
    }

    pub async fn offer_create(&self) -> Result<RtcSessionDescriptionInit> {
        Ok(JsFuture::from(self.0.create_offer()).await?.into())
    }

    pub async fn answer_create(&self) -> Result<RtcSessionDescriptionInit> {
        Ok(JsFuture::from(self.0.create_answer()).await?.into())
    }

    pub async fn local_desc_set(&self, desc: RtcSessionDescriptionInit) -> Result<()> {
        JsFuture::from(self.0.set_local_description(&desc)).await?;
        Ok(())
    }

    pub async fn remote_desc_set(&self, desc: RtcSessionDescriptionInit) -> Result<()> {
        JsFuture::from(self.0.set_remote_description(&desc)).await?;
        Ok(())
    }

    pub async fn local_sdp(&self) -> Option<String> {
        self.0.local_description().map(|v| v.sdp())
    }

    // pub async fn remote_sdp(&self) -> Option<String> {
    //     self.0.remote_description().map(|v| v.sdp())
    // }

    pub fn connection_state(&self) -> RTCConnectionState {
        self.0.connection_state()
    }

    pub async fn wait_for_ice_gathering_complete(&self) {
        if !matches!(self.0.ice_gathering_state(), RtcIceGatheringState::Complete) {
            let (tx, rx) = oneshot::channel::<()>();
            let mut tx = Some(tx);
            let conn = self.0.clone();
            let callback = Closure::<dyn FnMut()>::new(move || {
                if matches!(conn.ice_gathering_state(), RtcIceGatheringState::Complete) {
                    if let Some(tx) = tx.take() {
                        let _ = tx.send(());
                    }
                }
            });
            self.0
                .set_onicegatheringstatechange(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
            let _ = rx.await;
        }
    }

    pub fn on_connection_state_change(&self, mut f: OnConnectionStateChangeHdlrFn) {
        let conn = self.0.clone();
        let callback = Closure::new(move || {
            spawn_local(f(conn.connection_state()));
        });
        self.0
            .set_onconnectionstatechange(Some(callback.as_ref().unchecked_ref()));
        callback.forget();
    }

    pub async fn close(&self) {
        self.0.close();
    }
}

impl RTCChannel {
    pub fn is_main(&self) -> bool {
        self.1
    }

    pub fn on_open<Fut>(&self, mut f: impl FnMut() -> Fut + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        leaking_channel_event_handler(
            |f| self.0.set_onopen(f),
            move |_: JsValue| {
                spawn_local(f());
            },
        );
        // let callback = Closure::new::<dyn FnMut(_)>(move || {
        //     spawn_local(f());
        // });
        // self.0
        //     .set_onopen(Some(callback.as_ref().unchecked_ref()));
        // callback.forget();
    }

    pub fn on_message<Fut>(&self, mut f: impl FnMut(Vec<u8>) -> Fut + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        leaking_channel_event_handler(
            |f| self.0.set_onmessage(f),
            move |event: MessageEvent| {
                if let Ok(arraybuf) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let uarray = js_sys::Uint8Array::new(&arraybuf);
                    let data = uarray.to_vec();
                    spawn_local(f(data));
                } else {
                    openmina_core::log::error!(redux::Timestamp::global_now(); "`event.data()` failed to cast to `ArrayBuffer`. {:?}", event.data());
                }
            },
        );
    }

    pub fn on_error<Fut>(&self, mut f: impl FnMut(JsValue) -> Fut + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        //     pub fn on_error<Fut, Err>(&self, mut f: impl FnMut(Err) -> Fut + 'static)
        // where
        //     Fut: Future<Output = ()> + Send + 'static,
        //     Err: From<JsValue>,
        // {
        leaking_channel_event_handler(
            |f| self.0.set_onerror(f),
            move |err: JsValue| {
                spawn_local(f(err));
            },
        );
        // let callback = Closure::new::<dyn FnMut(_)>(move || {
        //     spawn_local(f());
        // });
        // self.0
        //     .set_onopen(Some(callback.as_ref().unchecked_ref()));
        // callback.forget();

        // self.0.on_error(Box::new(move |err| Box::pin(f(err))))
    }

    pub fn on_close<Fut>(&self, mut f: impl FnMut() -> Fut + 'static)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        leaking_channel_event_handler(
            |f| self.0.set_onclose(f),
            move |_: JsValue| {
                spawn_local(f());
            },
        );
    }

    pub async fn send(&self, data: &bytes::Bytes) -> Result<usize> {
        let len = data.len();
        let array = js_sys::Uint8Array::new_with_length(len as u32);
        array.copy_from(&data);
        self.0.send_with_array_buffer(&array.buffer()).map(|_| len)
    }

    pub async fn close(&self) {
        let _ = self.0.close();
    }
}

pub async fn webrtc_signal_send(
    url: &str,
    offer: Offer,
) -> std::result::Result<P2pConnectionResponse, RTCSignalingError> {
    use web_sys::{Request, RequestInit, Response};

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.body(Some(&JsValue::from(serde_json::to_string(&offer)?)));
    let request = Request::new_with_str_and_init(url, &opts)?;
    request.headers().set("content-type", "application/json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();
    let json = JsFuture::from(resp.json()?).await?;

    Ok(json.into_serde()?)
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

impl From<RTCConfig> for RtcConfiguration {
    fn from(value: RTCConfig) -> Self {
        let mut config = Self::new();
        config
            .ice_servers(&JsValue::from_serde(&value.ice_servers).unwrap())
            .ice_transport_policy(RtcIceTransportPolicy::All);
        config
    }
}

impl From<&RTCChannelConfig> for RtcDataChannelInit {
    fn from(value: &RTCChannelConfig) -> Self {
        let mut config = Self::new();

        if let Some(negotiated) = value.negotiated {
            config.negotiated(true).id(negotiated);
        }

        config
    }
}

impl TryFrom<Offer> for RtcSessionDescriptionInit {
    type Error = JsValue;

    fn try_from(value: Offer) -> Result<Self> {
        let mut sdp = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        sdp.sdp(&value.sdp);
        Ok(sdp)
    }
}

impl TryFrom<Answer> for RtcSessionDescriptionInit {
    type Error = JsValue;

    fn try_from(value: Answer) -> Result<Self> {
        let mut sdp = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        sdp.sdp(&value.sdp);
        Ok(sdp)
    }
}

/// Copied from https://github.com/johanhelsing/matchbox/blob/main/matchbox_socket/src/webrtc_socket/wasm.rs
///
/// Note that this function leaks some memory because the rust closure is dropped but still needs to
/// be accessed by javascript of the browser
///
/// See also: https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#method.into_js_value
fn leaking_channel_event_handler<T: FromWasmAbi + 'static>(
    mut setter: impl FnMut(Option<&js_sys::Function>),
    handler: impl FnMut(T) + 'static,
) {
    let closure: Closure<dyn FnMut(T)> = Closure::wrap(Box::new(handler));

    setter(Some(closure.as_ref().unchecked_ref()));

    closure.forget();
}

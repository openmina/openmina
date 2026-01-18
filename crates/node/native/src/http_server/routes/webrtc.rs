//! WebRTC signaling endpoints (feature-gated).
//!
//! - `GET /mina/webrtc/signal/{offer}` - Handle WebRTC signaling (GET with base58 encoded offer)
//! - `POST /mina/webrtc/signal` - Handle WebRTC signaling (POST with JSON offer)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use mina_node::{
    p2p::{
        connection::{
            incoming::{IncomingSignalingMethod, P2pConnectionIncomingInitOpts},
            P2pConnectionResponse,
        },
        webrtc, PeerId,
    },
    rpc::RpcRequest,
};
use mina_node_common::rpc::RpcP2pConnectionIncomingResponse;

use crate::http_server::AppState;

/// Registers WebRTC routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/mina/webrtc/signal/{offer}", get(signal_get))
        .route("/mina/webrtc/signal", post(signal_post))
}

/// Handles WebRTC signaling via GET with base58 encoded offer in path.
///
/// TODO(axum-migration): Returns 400 for both bad base58 AND bad JSON schema inside.
/// This matches warp behavior but differs from signal_post which returns 422 for
/// bad JSON schema. Could split: 400 for bad base58, 422 for bad JSON schema.
async fn signal_get(
    State(state): State<AppState>,
    Path(offer): Path<String>,
) -> (StatusCode, Json<P2pConnectionResponse>) {
    let decode_result = bs58::decode(&offer)
        .into_vec()
        .ok()
        .and_then(|json| serde_json::from_slice(&json).ok());

    match decode_result {
        None => (
            StatusCode::BAD_REQUEST,
            Json(P2pConnectionResponse::SignalDecryptionFailed),
        ),
        Some(offer) => handle_offer(state, offer).await,
    }
}

/// Handles WebRTC signaling via POST with JSON offer in body.
///
/// TODO(axum-migration): Malformed JSON returns 422 (axum default) vs warp's 400.
/// Both are framework defaults, not explicit choices. 422 is arguably more correct
/// (valid JSON, wrong schema = "unprocessable entity"). Noted for awareness.
async fn signal_post(
    State(state): State<AppState>,
    Json(offer): Json<Box<webrtc::Offer>>,
) -> (StatusCode, Json<P2pConnectionResponse>) {
    handle_offer(state, offer).await
}

/// Shared handler for processing WebRTC offers.
async fn handle_offer(
    state: AppState,
    offer: Box<webrtc::Offer>,
) -> (StatusCode, Json<P2pConnectionResponse>) {
    let mut rx = state
        .rpc_sender()
        .multishot_request(
            2,
            RpcRequest::P2pConnectionIncoming(P2pConnectionIncomingInitOpts {
                peer_id: PeerId::from_public_key(offer.identity_pub_key.clone()),
                signaling: IncomingSignalingMethod::Http,
                offer,
            }),
        )
        .await;

    match rx.recv().await {
        Some(RpcP2pConnectionIncomingResponse::Answer(answer)) => {
            let status = match &answer {
                P2pConnectionResponse::Accepted(_) => StatusCode::OK,
                P2pConnectionResponse::Rejected(reason) => {
                    if reason.is_bad() {
                        StatusCode::BAD_REQUEST
                    } else {
                        StatusCode::OK
                    }
                }
                P2pConnectionResponse::SignalDecryptionFailed => StatusCode::BAD_REQUEST,
                P2pConnectionResponse::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(answer))
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(P2pConnectionResponse::InternalError),
        ),
    }
}

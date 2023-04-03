use warp::{hyper::StatusCode, reply::with_status, Filter};

use snarker::{
    p2p::{
        connection::{
            incoming::{IncomingSignalingMethod, P2pConnectionIncomingInitOpts},
            P2pConnectionResponse,
        },
        webrtc, PeerId,
    },
    rpc::RpcRequest,
};

use super::rpc::RpcP2pConnectionIncomingResponse;

pub async fn run(port: u16, rpc_sender: super::RpcSender) {
    let signaling = warp::path!("mina" / "webrtc" / "signal")
        .and(warp::post())
        .and(warp::filters::body::json())
        .then(move |offer: webrtc::Offer| {
            let rpc_sender = rpc_sender.clone();
            async move {
                let mut rx = rpc_sender
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
                            P2pConnectionResponse::Rejected(reason) => match reason.is_bad() {
                                false => StatusCode::OK,
                                true => StatusCode::BAD_REQUEST,
                            },
                            P2pConnectionResponse::InternalError => {
                                StatusCode::INTERNAL_SERVER_ERROR
                            }
                        };
                        let resp = serde_json::to_string(&answer).unwrap_or_else(|_| {
                            P2pConnectionResponse::internal_error_json_str().to_owned()
                        });
                        with_status(resp, status)
                    }
                    _ => {
                        let resp = P2pConnectionResponse::internal_error_json_str().to_owned();
                        with_status(resp, StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
        });
    let routes = signaling;
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

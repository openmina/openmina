use std::str::FromStr;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use warp::{hyper::StatusCode, reply::with_status, Filter};

use shared::snark_job_id::SnarkJobId;
use snarker::{
    p2p::{
        connection::{
            incoming::{IncomingSignalingMethod, P2pConnectionIncomingInitOpts},
            P2pConnectionResponse,
        },
        webrtc, PeerId,
    },
    rpc::{ActionStatsQuery, RpcRequest, SyncStatsQuery},
};

use super::rpc::{
    RpcActionStatsGetResponse, RpcP2pConnectionIncomingResponse, RpcSnarkPoolGetResponse,
    RpcSnarkerJobCommitResponse, RpcStateGetResponse, RpcSyncStatsGetResponse,
};

pub async fn run(port: u16, rpc_sender: super::RpcSender) {
    let rpc_sender_clone = rpc_sender.clone();
    let signaling = warp::path!("mina" / "webrtc" / "signal")
        .and(warp::post())
        .and(warp::filters::body::json())
        .then(move |offer: webrtc::Offer| {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                let mut rx = rpc_sender_clone
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
                        with_json_reply(&answer, status)
                    }
                    _ => {
                        let resp = P2pConnectionResponse::internal_error_str();
                        with_json_reply(&resp, StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
        });

    // TODO(binier): make endpoint only accessible locally.
    let rpc_sender_clone = rpc_sender.clone();
    let state_get = warp::path!("state").and(warp::get()).then(move || {
        let rpc_sender_clone = rpc_sender_clone.clone();
        async move {
            let result: Option<RpcStateGetResponse> =
                rpc_sender_clone.oneshot_request(RpcRequest::GetState).await;

            with_json_reply(&result, StatusCode::OK)
        }
    });

    // TODO(binier): make endpoint only accessible locally.
    let stats = {
        let rpc_sender_clone = rpc_sender.clone();
        #[derive(Deserialize, Default)]
        struct ActionQueryParams {
            id: Option<String>,
        }
        let action_stats = warp::path!("stats" / "actions")
            .and(warp::get())
            .and(optq::<ActionQueryParams>())
            .then(move |query: ActionQueryParams| {
                let rpc_sender_clone = rpc_sender_clone.clone();
                async move {
                    let id_filter = query.id.as_ref().map(|s| s.as_str());
                    let result: RpcActionStatsGetResponse = rpc_sender_clone
                        .oneshot_request(RpcRequest::ActionStatsGet(match id_filter {
                            None => ActionStatsQuery::SinceStart,
                            Some("latest") => ActionStatsQuery::ForLatestBlock,
                            Some(id) => {
                                let id = match id.parse() {
                                    Ok(v) => v,
                                    Err(err) => {
                                        return with_json_reply(
                                            &format!(
                                                "'id' must be an u64 integer: {err}, instead passed: {id}"
                                            ),
                                            StatusCode::BAD_REQUEST,
                                        );
                                    }
                                };
                                ActionStatsQuery::ForBlockWithId(id)
                            }
                        }))
                        .await
                        .flatten();

                    with_json_reply(&result, StatusCode::OK)
                }
            });

        let rpc_sender_clone = rpc_sender.clone();
        #[derive(Deserialize, Default)]
        struct SyncQueryParams {
            limit: Option<usize>,
        }
        let sync_stats = warp::path!("stats" / "sync")
            .and(warp::get())
            .and(optq::<SyncQueryParams>())
            .then(move |query: SyncQueryParams| {
                let rpc_sender_clone = rpc_sender_clone.clone();
                async move {
                    let result: RpcSyncStatsGetResponse = rpc_sender_clone
                        .oneshot_request(RpcRequest::SyncStatsGet(SyncStatsQuery {
                            limit: query.limit,
                        }))
                        .await
                        .flatten();

                    with_json_reply(&result, StatusCode::OK)
                }
            });

        action_stats.or(sync_stats)
    };

    let rpc_sender_clone = rpc_sender.clone();
    let snark_pool_jobs_get = warp::path!("snark-pool" / "jobs")
        .and(warp::get())
        .then(move || {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                let res: Option<RpcSnarkPoolGetResponse> = rpc_sender_clone
                    .oneshot_request(RpcRequest::SnarkPoolGet)
                    .await;
                match res {
                    None => with_json_reply(
                        &"response channel dropped",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ),
                    Some(resp) => with_json_reply(&resp, StatusCode::OK),
                }
            }
        });

    // TODO(binier): make endpoint only accessible locally.
    let rpc_sender_clone = rpc_sender.clone();
    let snarker_job_commit = warp::path!("snarker" / "job" / "commit")
        .and(warp::put())
        .and(warp::filters::body::bytes())
        .then(move |body: bytes::Bytes| {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                let Ok(job_id) = String::from_utf8(body.to_vec())
                    .or(Err(()))
                    .and_then(|s| SnarkJobId::from_str(&s).or(Err(())))
                    else {
                        return with_json_reply(&"invalid_input", StatusCode::BAD_REQUEST);
                    };

                let res: Option<RpcSnarkerJobCommitResponse> = rpc_sender_clone
                    .oneshot_request(RpcRequest::SnarkerJobCommit { job_id })
                    .await;
                match res {
                    None => with_json_reply(
                        &"response channel dropped",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ),
                    Some(resp) => {
                        let status = match &resp {
                            RpcSnarkerJobCommitResponse::Ok => StatusCode::CREATED,
                            _ => StatusCode::BAD_REQUEST,
                        };
                        with_json_reply(&resp, status)
                    }
                }
            }
        });

    let cors = warp::cors().allow_any_origin();
    let routes = signaling
        .or(state_get)
        .or(stats)
        .or(snark_pool_jobs_get)
        .or(snarker_job_commit)
        .with(cors);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

use warp::filters::BoxedFilter;
use warp::reply::{json, Json, WithStatus};

fn optq<T: 'static + Default + Send + DeserializeOwned>() -> BoxedFilter<(T,)> {
    warp::any()
        .and(warp::query().or(warp::any().map(|| T::default())))
        .unify()
        .boxed()
}

fn with_json_reply<T: Serialize>(reply: &T, status: StatusCode) -> WithStatus<Json> {
    with_status(json(reply), status)
}

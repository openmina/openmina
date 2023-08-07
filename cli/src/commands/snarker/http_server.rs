use std::{mem::size_of, str::FromStr};

use binprot::BinProtWrite;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use warp::{
    http::HeaderValue,
    hyper::{header::CONTENT_TYPE, Response, StatusCode},
    reply::with_status,
    Filter, Reply,
};

use shared::snark_job_id::SnarkJobId;
use snarker::{
    p2p::{
        connection::{
            incoming::{IncomingSignalingMethod, P2pConnectionIncomingInitOpts},
            P2pConnectionResponse,
        },
        webrtc, PeerId,
    },
    rpc::{
        ActionStatsQuery, RpcRequest, RpcSnarkPoolJobGetResponse, RpcSnarkerWorkersResponse,
        SyncStatsQuery,
    },
};

use super::rpc::{
    RpcActionStatsGetResponse, RpcP2pConnectionIncomingResponse, RpcSnarkPoolGetResponse,
    RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse, RpcStateGetResponse,
    RpcSyncStatsGetResponse,
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

    let rpc_sender_clone = rpc_sender.clone();
    let snark_pool_job_get = warp::path!("snark-pool" / "job" / SnarkJobId).then(move |job_id| {
        let rpc_sender_clone = rpc_sender_clone.clone();
        async move {
            let res: Option<RpcSnarkPoolJobGetResponse> = rpc_sender_clone
                .oneshot_request(RpcRequest::SnarkPoolJobGet { job_id })
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
        .and(warp::post())
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

    #[derive(Deserialize)]
    struct JobIdParam {
        id: SnarkJobId,
    }

    let rpc_sender_clone = rpc_sender.clone();
    let snarker_job_spec = warp::path!("snarker" / "job" / "spec")
        .and(warp::get())
        .and(warp::header::optional("accept"))
        .and(warp::query())
        .then(
            move |accept: Option<String>, JobIdParam { id: job_id }: JobIdParam| {
                let rpc_sender_clone = rpc_sender_clone.clone();
                async move {
                    rpc_sender_clone
                        .oneshot_request(RpcRequest::SnarkerJobSpec { job_id })
                        .await
                        .map_or_else(
                            || {
                                JsonOrBinary::error(
                                    "response channel dropped",
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                )
                            },
                            |resp| match resp {
                                RpcSnarkerJobSpecResponse::Ok(spec)
                                    if accept.as_ref().map(String::as_str)
                                        == Some("application/octet-stream") =>
                                {
                                    JsonOrBinary::binary(spec)
                                }
                                RpcSnarkerJobSpecResponse::Ok(spec) => JsonOrBinary::json(spec),
                                _ => JsonOrBinary::error("error", StatusCode::BAD_REQUEST),
                            },
                        )
                }
            },
        );

    let dropped_channel_response = || {
        with_json_reply(
            &"response channel dropped",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    };

    let rpc_sender_clone = rpc_sender.clone();
    let snark_workers = warp::path!("snarker" / "workers")
        .and(warp::get())
        .then(move || {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                rpc_sender_clone
                    .oneshot_request(RpcRequest::SnarkerWorkers)
                    .await
                    .map_or_else(
                        dropped_channel_response,
                        |reply: RpcSnarkerWorkersResponse| with_json_reply(&reply, StatusCode::OK),
                    )
            }
        });

    let cors = warp::cors().allow_any_origin();
    let routes = signaling
        .or(state_get)
        .or(stats)
        .or(snark_pool_jobs_get)
        .or(snark_pool_job_get)
        .or(snarker_job_commit)
        .or(snarker_job_spec)
        .or(snark_workers)
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

pub enum JsonOrBinary {
    Json(Vec<u8>),
    Binary(Vec<u8>),
    Error(String, StatusCode),
}

impl JsonOrBinary {
    fn json<T: Serialize>(reply: T) -> Self {
        match serde_json::to_vec(&reply) {
            Ok(v) => JsonOrBinary::Json(v),
            Err(err) => JsonOrBinary::error(err, StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
    fn binary<T: BinProtWrite>(reply: T) -> Self {
        let mut vec = Vec::new();
        match reply.binprot_write(&mut vec) {
            Ok(()) => {}
            Err(err) => return JsonOrBinary::error(err, StatusCode::INTERNAL_SERVER_ERROR),
        }
        let mut result = Vec::with_capacity(vec.len() + size_of::<u64>());
        result.extend((vec.len() as u64).to_le_bytes());
        result.extend(vec);
        JsonOrBinary::Binary(result)
    }
    fn error<T: ToString>(err: T, code: StatusCode) -> Self {
        JsonOrBinary::Error(err.to_string(), code)
    }
}

impl Reply for JsonOrBinary {
    fn into_response(self) -> warp::reply::Response {
        match self {
            JsonOrBinary::Json(body) => {
                let mut res = Response::new(body.into());
                res.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                *res.status_mut() = StatusCode::OK;
                res
            }
            JsonOrBinary::Binary(body) => {
                let mut res = Response::new(body.into());
                res.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/octet-stream"),
                );
                *res.status_mut() = StatusCode::OK;
                res
            }
            JsonOrBinary::Error(err, code) => {
                let mut res = Response::new(err.into());
                res.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("plain/text"));
                *res.status_mut() = code;
                res
            }
        }
    }
}

fn with_json_reply<T: Serialize>(reply: &T, status: StatusCode) -> WithStatus<Json> {
    with_status(json(reply), status)
}

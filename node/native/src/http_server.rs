use std::{convert::Infallible, mem::size_of, str::FromStr};

use mina_p2p_messages::binprot::BinProtWrite;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use warp::{
    http::HeaderValue,
    hyper::{header::CONTENT_TYPE, Response, StatusCode},
    reply::with_status,
    Filter, Rejection, Reply,
};

use node::core::snark::SnarkJobId;
use node::rpc::{
    ActionStatsQuery, RpcBlockProducerStatsGetResponse, RpcMessageProgressResponse, RpcPeerInfo,
    RpcRequest, RpcScanStateSummaryGetQuery, RpcScanStateSummaryGetResponse,
    RpcSnarkPoolJobGetResponse, RpcSnarkerWorkersResponse, RpcStateGetError, RpcStatusGetResponse,
    SyncStatsQuery,
};

use openmina_node_common::rpc::{
    RpcActionStatsGetResponse, RpcSender, RpcSnarkPoolGetResponse, RpcSnarkerJobCommitResponse,
    RpcSnarkerJobSpecResponse, RpcStateGetResponse, RpcSyncStatsGetResponse,
};

pub async fn run(port: u16, rpc_sender: RpcSender) {
    #[cfg(feature = "p2p-webrtc")]
    let signaling = {
        use node::p2p::{
            connection::{
                incoming::{IncomingSignalingMethod, P2pConnectionIncomingInitOpts},
                P2pConnectionResponse,
            },
            webrtc, PeerId,
        };

        use super::rpc::RpcP2pConnectionIncomingResponse;

        let rpc_sender_clone = rpc_sender.clone();
        warp::path!("mina" / "webrtc" / "signal")
            .and(warp::post())
            .and(warp::filters::body::json())
            .then(move |offer: Box<webrtc::Offer>| {
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
            })
    };

    // TODO(binier): make endpoint only accessible locally.
    #[derive(Deserialize)]
    struct StateQueryParams {
        filter: Option<String>,
    }
    #[derive(Debug)]
    struct StateGetRejection(RpcStateGetError);
    impl warp::reject::Reject for StateGetRejection {}

    let state_get = warp::path!("state")
        .and(warp::get())
        .and(with_rpc_sender(rpc_sender.clone()))
        .and(warp::query())
        .and_then(state_handler)
        .recover(state_recover);

    let state_post = warp::path!("state")
        .and(warp::post())
        .and(with_rpc_sender(rpc_sender.clone()))
        .and(warp::body::json())
        .and_then(state_handler)
        .recover(state_recover);

    async fn state_handler(
        rpc_sender: RpcSender,
        StateQueryParams { filter }: StateQueryParams,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        rpc_sender
            .oneshot_request(RpcRequest::StateGet(filter))
            .await
            .ok_or_else(|| warp::reject::custom(DroppedChannel))
            .and_then(|reply: RpcStateGetResponse| {
                reply.map_or_else(
                    |err| Err(warp::reject::custom(StateGetRejection(err))),
                    |state| Ok(warp::reply::json(&state)),
                )
            })
    }

    async fn state_recover(reject: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
        if let Some(StateGetRejection(error)) = reject.find() {
            Ok(warp::reply::with_status(
                warp::reply::json(error),
                StatusCode::BAD_REQUEST,
            ))
        } else {
            Err(reject)
        }
    }

    let rpc_sender_clone = rpc_sender.clone();
    let status = warp::path!("status").and(warp::get()).then(move || {
        let rpc_sender_clone = rpc_sender_clone.clone();
        async move {
            let result: RpcStatusGetResponse = rpc_sender_clone
                .oneshot_request(RpcRequest::StatusGet)
                .await
                .flatten();

            with_json_reply(&result, StatusCode::OK)
        }
    });

    let rpc_sender_clone = rpc_sender.clone();
    let peers_get = warp::path!("state" / "peers")
        .and(warp::get())
        .then(move || {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                let result: Option<Vec<RpcPeerInfo>> =
                    rpc_sender_clone.oneshot_request(RpcRequest::PeersGet).await;

                with_json_reply(&result, StatusCode::OK)
            }
        });

    let rpc_sender_clone = rpc_sender.clone();
    let message_progress_get = warp::path!("state" / "message-progress")
        .and(warp::get())
        .then(move || {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                let result = rpc_sender_clone
                    .oneshot_request::<RpcMessageProgressResponse>(RpcRequest::MessageProgressGet)
                    .await;

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
                    let id_filter = query.id.as_deref();
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

        let rpc_sender_clone = rpc_sender.clone();
        let block_producer_stats = warp::path!("stats" / "block_producer")
            .and(warp::get())
            .then(move || {
                let rpc_sender_clone = rpc_sender_clone.clone();
                async move {
                    let result: RpcBlockProducerStatsGetResponse = rpc_sender_clone
                        .oneshot_request(RpcRequest::BlockProducerStatsGet)
                        .await
                        .flatten();

                    with_json_reply(&result, StatusCode::OK)
                }
            });

        action_stats.or(sync_stats).or(block_producer_stats)
    };

    let rpc_sender_clone = rpc_sender.clone();
    let scan_state_summary_get = warp::path!("scan-state" / "summary" / ..)
        .and(warp::get())
        .and(
            warp::path::param::<String>()
                .map(Some)
                .or_else(|_| async { Ok::<(Option<String>,), std::convert::Infallible>((None,)) }),
        )
        .and(warp::path::end())
        .then(move |query: Option<String>| {
            let rpc_sender_clone = rpc_sender_clone.clone();
            let query = match query {
                None => Ok(RpcScanStateSummaryGetQuery::ForBestTip),
                Some(query) => None
                    .or_else(|| {
                        Some(RpcScanStateSummaryGetQuery::ForBlockWithHeight(
                            query.parse().ok()?,
                        ))
                    })
                    .ok_or(())
                    .or_else(|_| match query.parse() {
                        Err(_) => Err("invalid arg! Expected block hash or height"),
                        Ok(v) => Ok(RpcScanStateSummaryGetQuery::ForBlockWithHash(v)),
                    }),
            };
            async move {
                let query = match query {
                    Ok(v) => v,
                    Err(err) => {
                        return with_json_reply(&err, StatusCode::BAD_REQUEST);
                    }
                };
                let res: Option<RpcScanStateSummaryGetResponse> = rpc_sender_clone
                    .oneshot_request(RpcRequest::ScanStateSummaryGet(query))
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
                                    if accept.as_deref() == Some("application/octet-stream") =>
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

    let rpc_sender_clone = rpc_sender.clone();
    let snarker_config = warp::path!("snarker" / "config")
        .and(warp::get())
        .then(move || {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                rpc_sender_clone
                    .oneshot_request(RpcRequest::SnarkerConfig)
                    .await
                    .map_or_else(
                        dropped_channel_response,
                        |reply: node::rpc::RpcSnarkerConfigGetResponse| {
                            with_json_reply(&reply, StatusCode::OK)
                        },
                    )
            }
        });

    let rpc_sender_clone = rpc_sender.clone();
    let transaction_pool = warp::path!("transaction-pool")
        .and(warp::get())
        .then(move || {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                rpc_sender_clone
                    .oneshot_request(RpcRequest::TransactionPoolGet)
                    .await
                    .map_or_else(
                        dropped_channel_response,
                        |reply: node::rpc::RpcTransactionPoolResponse| {
                            with_json_reply(&reply, StatusCode::OK)
                        },
                    )
            }
        });

    let rpc_sender_clone = rpc_sender.clone();
    let accounts = warp::path("accounts").and(warp::get()).then(move || {
        let rpc_sender_clone = rpc_sender_clone.clone();

        async move {
            rpc_sender_clone
                .oneshot_request(RpcRequest::LedgerAccountsGet(None))
                .await
                .map_or_else(
                    dropped_channel_response,
                    |reply: node::rpc::RpcLedgerAccountsResponse| {
                        with_json_reply(&reply, StatusCode::OK)
                    },
                )
        }
    });

    let rpc_sender_clone = rpc_sender.clone();
    let transaction_post = warp::path("send-payment")
        .and(warp::post())
        .and(warp::filters::body::json())
        .then(move |body: Vec<_>| {
            let rpc_sender_clone = rpc_sender_clone.clone();

            async move {
                println!("Transaction inject post: {:#?}", body);
                rpc_sender_clone
                    .oneshot_request(RpcRequest::TransactionInject(body))
                    .await
                    .map_or_else(
                        dropped_channel_response,
                        |reply: node::rpc::RpcTransactionInjectResponse| {
                            with_json_reply(&reply, StatusCode::OK)
                        },
                    )
            }
        });

    let rpc_sender_clone = rpc_sender.clone();
    let transition_frontier_user_commands = warp::path("best-chain-user-commands")
        .and(warp::get())
        .then(move || {
            let rpc_sender_clone = rpc_sender_clone.clone();

            async move {
                rpc_sender_clone
                    .oneshot_request(RpcRequest::TransitionFrontierUserCommandsGet)
                    .await
                    .map_or_else(
                        dropped_channel_response,
                        |reply: node::rpc::RpcTransitionFrontierUserCommandsResponse| {
                            with_json_reply(&reply, StatusCode::OK)
                        },
                    )
            }
        });

    let cors = warp::cors().allow_any_origin();
    #[cfg(not(feature = "p2p-webrtc"))]
    let routes = state_get.or(state_post);
    #[cfg(feature = "p2p-webrtc")]
    let routes = signaling.or(state_get).or(state_post);
    let routes = routes
        .or(status)
        .or(peers_get)
        .or(message_progress_get)
        .or(stats)
        .or(scan_state_summary_get)
        .or(snark_pool_jobs_get)
        .or(snark_pool_job_get)
        .or(snarker_config)
        .or(snarker_job_commit)
        .or(snarker_job_spec)
        .or(snark_workers)
        .or(transaction_pool)
        .or(accounts)
        .or(transaction_post)
        .boxed()
        .or(transition_frontier_user_commands)
        .boxed()
        .or(healthcheck(rpc_sender.clone()))
        .or(readiness(rpc_sender.clone()))
        .or(discovery::routing_table(rpc_sender.clone()))
        .or(discovery::bootstrap_stats(rpc_sender.clone()))
        .or(super::graphql::routes(rpc_sender))
        .recover(recover)
        .with(cors);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

fn healthcheck(
    rpc_sender: RpcSender,
) -> impl Filter<Error = Rejection, Extract = impl Reply> + Clone {
    warp::path!("healthz").and(warp::get()).then(move || {
        let rpc_sender = rpc_sender.clone();
        async move {
            rpc_sender
                .oneshot_request(RpcRequest::HealthCheck)
                .await
                .map_or_else(
                    || {
                        with_status(
                            String::from(DROPPED_CHANNEL),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    },
                    |reply: node::rpc::RpcHealthCheckResponse| match reply {
                        Ok(()) => with_status(String::new(), StatusCode::OK),
                        Err(err) => with_status(err, StatusCode::SERVICE_UNAVAILABLE),
                    },
                )
        }
    })
}

fn readiness(
    rpc_sender: RpcSender,
) -> impl Filter<Error = Rejection, Extract = impl Reply> + Clone {
    warp::path!("readyz").and(warp::get()).then(move || {
        let rpc_sender = rpc_sender.clone();
        async move {
            rpc_sender
                .oneshot_request(RpcRequest::ReadinessCheck)
                .await
                .map_or_else(
                    || {
                        with_status(
                            String::from(DROPPED_CHANNEL),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    },
                    |reply: node::rpc::RpcReadinessCheckResponse| match reply {
                        Ok(()) => with_status(String::new(), StatusCode::OK),
                        Err(err) => with_status(err, StatusCode::SERVICE_UNAVAILABLE),
                    },
                )
        }
    })
}

mod discovery {
    use node::rpc::{
        RpcDiscoveryBoostrapStatsResponse, RpcDiscoveryRoutingTableResponse, RpcRequest,
    };
    use openmina_node_common::rpc::RpcSender;
    use warp::Filter;

    use super::{with_rpc_sender, DroppedChannel};

    pub fn routing_table(
        rpc_sender: RpcSender,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("discovery" / "routing_table")
            .and(warp::get())
            .and(with_rpc_sender(rpc_sender))
            .and_then(get_routing_table)
    }

    pub fn bootstrap_stats(
        rpc_sender: RpcSender,
    ) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("discovery" / "bootstrap_stats")
            .and(warp::get())
            .and(with_rpc_sender(rpc_sender))
            .and_then(get_bootstrap_stats)
    }

    async fn get_routing_table(rpc_sender: RpcSender) -> Result<impl warp::Reply, warp::Rejection> {
        rpc_sender
            .oneshot_request(RpcRequest::DiscoveryRoutingTable)
            .await
            .map_or_else(
                || Err(warp::reject::custom(DroppedChannel)),
                |reply: RpcDiscoveryRoutingTableResponse| Ok(warp::reply::json(&reply)),
            )
    }

    async fn get_bootstrap_stats(
        rpc_sender: RpcSender,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        rpc_sender
            .oneshot_request(RpcRequest::DiscoveryBoostrapStats)
            .await
            .map_or_else(
                || Err(warp::reject::custom(DroppedChannel)),
                |reply: RpcDiscoveryBoostrapStatsResponse| Ok(warp::reply::json(&reply)),
            )
    }
}

fn with_rpc_sender(
    rpc_sender: RpcSender,
) -> impl warp::Filter<Extract = (RpcSender,), Error = Infallible> + Clone {
    warp::any().map(move || rpc_sender.clone())
}

const DROPPED_CHANNEL: &str = "response channel dropped, see error log for details";

#[derive(Debug)]
struct DroppedChannel;

impl warp::reject::Reject for DroppedChannel {}

async fn recover(rejection: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(DroppedChannel) = rejection.find() {
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": DROPPED_CHANNEL})),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else {
        Err(rejection)
    }
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

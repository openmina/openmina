use warp::{hyper::StatusCode, reply::with_status, Filter};

use snarker::{
    p2p::{
        channels::snark_job_commitment::{
            LedgerHashTransition, LedgerHashTransitionPasses, SnarkJobId,
        },
        connection::{
            incoming::{IncomingSignalingMethod, P2pConnectionIncomingInitOpts},
            P2pConnectionResponse,
        },
        webrtc, PeerId,
    },
    rpc::RpcRequest,
};

use super::rpc::{RpcP2pConnectionIncomingResponse, RpcSnarkerJobPickAndCommitResponse};

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

    // TODO(binier): make endpoint only accessible locally.
    let rpc_sender_clone = rpc_sender.clone();
    let snarker_pick_job = warp::path!("snarker" / "pick-job")
        .and(warp::put())
        .and(warp::filters::body::bytes())
        .then(move |body: bytes::Bytes| {
            let rpc_sender_clone = rpc_sender_clone.clone();
            async move {
                let Ok(s) = String::from_utf8(body.to_vec()) else {
                    return with_status("invalid input".to_owned(), StatusCode::BAD_REQUEST);
                };
                let jobs_res = s
                    .lines()
                    .map(|s| parse_snark_job_id(s))
                    .collect::<Result<Vec<_>, _>>();

                match jobs_res {
                    Ok(available_jobs) => {
                        let res: Option<RpcSnarkerJobPickAndCommitResponse> = rpc_sender_clone
                            .oneshot_request(RpcRequest::SnarkerJobPickAndCommit { available_jobs })
                            .await;
                        match res.flatten() {
                            None => with_status("".to_owned(), StatusCode::OK),
                            Some(job_id) => {
                                with_status(job_id_to_string(&job_id), StatusCode::CREATED)
                            }
                        }
                    }
                    Err(_) => with_status("invalid input".to_owned(), StatusCode::BAD_REQUEST),
                }
            }
        });

    let routes = signaling.or(snarker_pick_job);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

fn job_id_to_string(v: &SnarkJobId) -> String {
    match v {
        SnarkJobId::One(v) => ledger_hash_transition_to_string(v),
        SnarkJobId::Two(one, two) => format!(
            "{}::{}",
            ledger_hash_transition_to_string(one),
            ledger_hash_transition_to_string(two)
        ),
    }
}

fn ledger_hash_transition_to_string(v: &LedgerHashTransition) -> String {
    format!(
        "{}->{}:{}->{}",
        v.source.first_pass_ledger,
        v.target.first_pass_ledger,
        v.source.second_pass_ledger,
        v.target.second_pass_ledger
    )
}

fn parse_snark_job_id(s: &str) -> Result<SnarkJobId, ()> {
    Ok(match s.split_once("::") {
        None => SnarkJobId::One(parse_ledger_hash_transition(s)?),
        Some((one, two)) => SnarkJobId::Two(
            parse_ledger_hash_transition(one)?,
            parse_ledger_hash_transition(two)?,
        ),
    })
}

fn parse_ledger_hash_transition(s: &str) -> Result<LedgerHashTransition, ()> {
    let (first, second) = s.split_once(':').ok_or(())?;

    let (source_first, target_first) = first.split_once("->").ok_or(())?;
    let (source_second, target_second) = second.split_once("->").ok_or(())?;

    Ok(LedgerHashTransition {
        source: LedgerHashTransitionPasses {
            first_pass_ledger: source_first.parse().or(Err(()))?,
            second_pass_ledger: source_second.parse().or(Err(()))?,
        },
        target: LedgerHashTransitionPasses {
            first_pass_ledger: target_first.parse().or(Err(()))?,
            second_pass_ledger: target_second.parse().or(Err(()))?,
        },
    })
}

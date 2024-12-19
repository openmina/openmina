pub mod simulator;
pub mod webnode;

use crate::cluster::{Cluster, ClusterConfig, ClusterNodeId};
use crate::node::NodeTestingConfig;
use crate::scenario::{event_details, Scenario, ScenarioId, ScenarioInfo, ScenarioStep};
use crate::service::PendingEventId;

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::{collections::BTreeMap, sync::Arc, time::Duration};

use axum::http::header;
use axum::middleware;
use axum::routing::get_service;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use node::account::AccountPublicKey;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use node::p2p::webrtc::{Host, Offer, P2pConnectionResponse, SignalingMethod};
use node::transition_frontier::genesis::{GenesisConfig, PrebuiltGenesisConfig};
use openmina_node_native::p2p::webrtc::webrtc_signal_send;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::sync::{oneshot, Mutex, MutexGuard, OwnedMutexGuard};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub fn server(rt: Runtime, host: Host, port: u16, ssl_port: Option<u16>) {
    let fe_dist_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join("frontend/dist/frontend/");
    eprintln!("scenarios path: {}", Scenario::PATH);
    eprintln!("FrontEnd dist path: {fe_dist_dir:?}");

    let state = AppState::new(host, ssl_port);

    let scenarios_router = Router::new()
        .route("/", get(scenario_list))
        .route("/", put(scenario_create))
        .route("/:id", get(scenario_get))
        .route("/:id/nodes", put(scenario_node_add))
        .route("/:id/steps", put(scenario_step_add));

    let clusters_router = Router::new()
        .route("/", get(cluster_list))
        .route("/create/:scenario_id", put(cluster_create))
        .route("/:cluster_id", get(cluster_get))
        .nest("/:cluster_id/webnode", webnode::router())
        .route("/:cluster_id/run", post(cluster_run))
        .route("/:cluster_id/run/auto", post(cluster_run_auto))
        .route(
            "/:cluster_id/scenarios/reload",
            post(cluster_scenarios_reload),
        )
        .route(
            "/:cluster_id/mina/webrtc/signal/:offer",
            get(cluster_webrtc_signal),
        )
        .route("/:cluster_id/seeds", get(cluster_seeds))
        .route("/:cluster_id/genesis/config", get(cluster_genesis_config))
        .route(
            "/:cluster_id/nodes/events/pending",
            get(cluster_events_pending),
        )
        .route(
            "/:cluster_id/nodes/:node_id/events/pending",
            get(cluster_node_events_pending),
        )
        .route("/:cluster_id/destroy", post(cluster_destroy));

    let cors = CorsLayer::very_permissive();
    let coop_coep = middleware::from_fn(|req, next: middleware::Next| async {
        let mut resp = next.run(req).await;
        resp.headers_mut().insert(
            header::HeaderName::from_static("cross-origin-embedder-policy"),
            header::HeaderValue::from_static("require-corp"),
        );
        resp.headers_mut().insert(
            header::HeaderName::from_static("cross-origin-opener-policy"),
            header::HeaderValue::from_static("same-origin"),
        );
        resp
    });

    let app = Router::new()
        .nest("/scenarios", scenarios_router)
        .nest("/clusters", clusters_router)
        .nest("/simulations", simulator::simulations_router())
        .fallback(get_service(ServeDir::new(&fe_dist_dir)).layer(coop_coep.clone()))
        .with_state(state)
        .layer(cors);

    rt.block_on(async {
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });
}

pub struct AppStateInner {
    host: Host,
    ssl_port: Option<u16>,
    rng: StdRng,
    clusters: BTreeMap<u16, Arc<Mutex<Cluster>>>,
    // TODO(binier): move inside cluster state
    locked_block_producer_keys: BTreeMap<u16, BTreeSet<AccountPublicKey>>,
}

impl AppStateInner {
    pub fn new(host: Host, ssl_port: Option<u16>) -> Self {
        Self {
            host,
            ssl_port,
            rng: StdRng::seed_from_u64(0),
            clusters: Default::default(),
            locked_block_producer_keys: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct AppState(Arc<Mutex<AppStateInner>>);

impl AppState {
    pub fn new(host: Host, ssl_port: Option<u16>) -> Self {
        Self(Arc::new(Mutex::new(AppStateInner::new(host, ssl_port))))
    }

    pub async fn lock(&self) -> MutexGuard<'_, AppStateInner> {
        self.0.lock().await
    }

    async fn cluster_mutex(
        &self,
        cluster_id: u16,
    ) -> Result<Arc<Mutex<Cluster>>, (StatusCode, String)> {
        self.lock().await.cluster_mutex(cluster_id)
    }

    pub async fn cluster(
        &self,
        cluster_id: u16,
    ) -> Result<OwnedMutexGuard<Cluster>, (StatusCode, String)> {
        self.lock().await.cluster(cluster_id).await
    }

    pub async fn cluster_create(
        &self,
        scenario_id: ScenarioId,
        config: ClusterConfig,
    ) -> Result<(u16, OwnedMutexGuard<Cluster>), (StatusCode, String)> {
        let scenario = Scenario::load(&scenario_id)
            .await
            .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

        let mut cluster = Cluster::new(config);
        cluster
            .start(scenario)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        let mut state = self.lock().await;
        let id = loop {
            let id = state.rng.gen();
            if !state.clusters.contains_key(&id) {
                break id;
            }
        };

        let cluster = Arc::new(Mutex::new(cluster));
        let cluster_guard = cluster.clone().try_lock_owned().unwrap();
        state.clusters.insert(id, cluster);

        Ok((id, cluster_guard))
    }

    pub async fn cluster_create_empty(
        &self,
        config: ClusterConfig,
    ) -> Result<(u16, OwnedMutexGuard<Cluster>), (StatusCode, String)> {
        let cluster = Cluster::new(config);

        let mut state = self.lock().await;
        let id = loop {
            let id = state.rng.gen();
            if !state.clusters.contains_key(&id) {
                break id;
            }
        };

        let cluster = Arc::new(Mutex::new(cluster));
        let cluster_guard = cluster.clone().try_lock_owned().unwrap();
        state.clusters.insert(id, cluster);

        Ok((id, cluster_guard))
    }

    pub async fn cluster_destroy(&self, cluster_id: u16) -> bool {
        let mut this = self.lock().await;
        this.locked_block_producer_keys.remove(&cluster_id);
        this.clusters.remove(&cluster_id).is_some()
    }
}

impl AppStateInner {
    fn cluster_mutex(&self, cluster_id: u16) -> Result<Arc<Mutex<Cluster>>, (StatusCode, String)> {
        self.clusters.get(&cluster_id).cloned().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("cluster {cluster_id} not found"),
            )
        })
    }

    pub async fn cluster(
        &self,
        cluster_id: u16,
    ) -> Result<OwnedMutexGuard<Cluster>, (StatusCode, String)> {
        Ok(self.cluster_mutex(cluster_id)?.lock_owned().await)
    }
}

async fn scenario_list(
    State(_): State<AppState>,
) -> Result<Json<Vec<ScenarioInfo>>, (StatusCode, String)> {
    Scenario::list()
        .await
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn scenario_get(
    State(_): State<AppState>,
    Path(id): Path<ScenarioId>,
) -> Result<Json<Scenario>, (StatusCode, String)> {
    Scenario::load(&id)
        .await
        .map(Json)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))
}

#[derive(Deserialize)]
struct ScenariosCreateArgs {
    id: ScenarioId,
    description: String,
    parent_id: Option<ScenarioId>,
}

async fn scenario_create(
    State(_): State<AppState>,
    Json(args): Json<ScenariosCreateArgs>,
) -> Result<StatusCode, (StatusCode, String)> {
    if Scenario::exists(&args.id) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("scenario with same id/name already exists: {}", args.id),
        ));
    }
    let mut scenario = Scenario::new(args.id, args.parent_id);
    scenario.set_description(args.description);
    scenario
        .save()
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn scenario_node_add(
    State(_): State<AppState>,
    Path(id): Path<ScenarioId>,
    Json(config): Json<NodeTestingConfig>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut scenario = Scenario::load(&id)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    scenario.add_node(config);
    scenario
        .save()
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn scenario_step_add(
    State(_): State<AppState>,
    Path(id): Path<ScenarioId>,
    Json(step): Json<ScenarioStep>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut scenario = Scenario::load(&id)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    scenario
        .add_step(step)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    scenario
        .save()
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn cluster_list(State(state): State<AppState>) -> Json<Vec<u16>> {
    let state = state.lock().await;
    Json(state.clusters.keys().cloned().collect())
}

#[derive(Serialize)]
struct ClusterGetResponse {
    id: u16,
    target_scenario: Option<ScenarioId>,
    next: Option<ScenarioWithStep>,
}

#[derive(Serialize)]
struct ScenarioWithStep {
    scenario: ScenarioId,
    step: usize,
}

async fn cluster_get(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Result<Json<ClusterGetResponse>, (StatusCode, String)> {
    let cluster = state.cluster(cluster_id).await?;

    Ok(Json(ClusterGetResponse {
        id: cluster_id,
        target_scenario: cluster.target_scenario().cloned(),
        next: cluster
            .next_scenario_and_step()
            .map(|(scenario, step)| ScenarioWithStep {
                scenario: scenario.clone(),
                step,
            }),
    }))
}

#[derive(Serialize)]
struct ClusterCreateResponse {
    cluster_id: u16,
}

async fn cluster_create(
    State(state): State<AppState>,
    Path(scenario_id): Path<ScenarioId>,
    args: Option<Json<ClusterConfig>>,
) -> Result<Json<ClusterCreateResponse>, (StatusCode, String)> {
    let config = match args {
        Some(Json(v)) => v,
        None => ClusterConfig::new(None)
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?,
    };
    state
        .cluster_create(scenario_id, config)
        .await
        .map(|(cluster_id, _)| Json(ClusterCreateResponse { cluster_id }))
}

#[derive(Deserialize, Default)]
struct ClusterRunArgs {
    exec_until: Option<ClusterExecUntil>,
}

#[derive(Deserialize)]
struct ClusterExecUntil {
    scenario: ScenarioId,
    step: Option<usize>,
}

async fn cluster_run(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
    args: Option<Json<ClusterRunArgs>>,
) -> Result<(), (StatusCode, String)> {
    let Json(args) = args.unwrap_or_default();
    let mut cluster = state.cluster(cluster_id).await?;
    let res = match args.exec_until {
        None => cluster.exec_to_end().await,
        Some(until) => cluster.exec_until(until.scenario, until.step).await,
    };

    res.map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn cluster_run_auto(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Result<(), (StatusCode, String)> {
    let mut cluster = state.cluster(cluster_id).await?;

    let _ = cluster.target_scenario().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "target scenario for cluster isnt set".to_owned(),
        )
    })?;

    cluster
        .exec_to_end()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let (tx, rx) = oneshot::channel::<Result<(), String>>();

    tokio::spawn(async move {
        while !tx.is_closed() {
            let steps = cluster
                .pending_events(true)
                .flat_map(|(node_id, _, pending_events)| {
                    pending_events.map(move |(_, event)| ScenarioStep::Event {
                        node_id,
                        event: event.to_string(),
                    })
                })
                .collect::<Vec<_>>();

            if steps.is_empty() {
                if cluster
                    .wait_for_pending_events_with_timeout(Duration::from_secs(5))
                    .await
                {
                    continue;
                } else {
                    break;
                }
            }

            cluster.add_steps_and_save(steps).await;

            if let Err(err) = cluster.exec_to_end().await {
                let _ = tx.send(Err(err.to_string()));
                return;
            }
        }

        let _ = tx.send(Ok(()));
    });

    rx.await
        .unwrap()
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))
}

async fn cluster_scenarios_reload(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Result<(), (StatusCode, String)> {
    let mut cluster = state.cluster(cluster_id).await?;
    cluster
        .reload_scenarios()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn cluster_webrtc_signal(
    State(state): State<AppState>,
    Path((cluster_id, offer)): Path<(u16, String)>,
) -> Result<Json<P2pConnectionResponse>, (StatusCode, Json<P2pConnectionResponse>)> {
    let offer: Offer = Err(())
        .or_else(move |_| {
            let json = bs58::decode(&offer).into_vec().or(Err(()))?;
            serde_json::from_slice(&json).or(Err(()))
        })
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(P2pConnectionResponse::SignalDecryptionFailed),
            )
        })?;

    let internal_err = || {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(P2pConnectionResponse::InternalError),
        )
    };

    let http_url = {
        let cluster = state
            .cluster(cluster_id)
            .await
            .map_err(|_| internal_err())?;
        let node = cluster
            .node_by_peer_id(offer.target_peer_id)
            .ok_or_else(internal_err)?;
        let http_url = match node.dial_addr() {
            P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } => signaling.http_url(),
            _ => None,
        };
        http_url.ok_or_else(internal_err)?
    };
    let resp = webrtc_signal_send(&http_url, offer)
        .await
        .map_err(|_| internal_err())?;
    Ok(Json(resp))
}

async fn cluster_seeds(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Result<String, (StatusCode, String)> {
    let state = state.lock().await;
    let host = state.host.clone();
    let ssl_port = state.ssl_port;
    state.cluster(cluster_id).await.map(|cluster| {
        let list = cluster
            .nodes_iter()
            .filter(|(_, node)| node.config().initial_peers.is_empty())
            .map(|(_, node)| {
                let mut addr = node.dial_addr();
                if let P2pConnectionOutgoingInitOpts::WebRTC { signaling, .. } = &mut addr {
                    if let SignalingMethod::Http(http) = signaling {
                        if let Some(port) = ssl_port {
                            http.host = host.clone();
                            http.port = port;
                            *signaling = SignalingMethod::HttpsProxy(cluster_id, http.clone());
                        }
                    }
                }
                addr = addr.to_string().parse().unwrap();
                addr.to_string()
            })
            .collect::<Vec<_>>();
        list.join("\n")
    })
}

async fn cluster_genesis_config(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Result<Vec<u8>, (StatusCode, String)> {
    let cluster = state.cluster(cluster_id).await?;
    let genesis = cluster
        .nodes_iter()
        .next()
        .map(|(_, node)| node.config().genesis.clone());
    let genesis = genesis.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "no nodes in the cluster".to_owned(),
        )
    })?;
    if let GenesisConfig::Prebuilt(encoded) = &*genesis {
        return Ok(encoded.clone().into_owned());
    }
    tokio::task::spawn_blocking(move || {
        let res = genesis.load().map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load genesis config. err: {err}"),
            )
        })?;
        let mut encoded = Vec::new();
        PrebuiltGenesisConfig::from_loaded(res)
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to build `PrebuiltGenesisConfig` from loaded data".to_owned(),
                )
            })?
            .store(&mut encoded)
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to encode `PrebuiltGenesisConfig`".to_owned(),
                )
            })?;
        Ok(encoded)
    })
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "join error".to_owned()))?
}

#[derive(Serialize)]
struct ClusterNodePendingEvents {
    node_id: ClusterNodeId,
    pending_events: Vec<ClusterNodePendingEvent>,
}

#[derive(Serialize)]
struct ClusterNodePendingEvent {
    id: PendingEventId,
    event: String,
    details: Option<String>,
}

async fn cluster_events_pending(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Result<Json<Vec<ClusterNodePendingEvents>>, (StatusCode, String)> {
    state
        .cluster(cluster_id)
        .await
        .map(|mut cluster| {
            cluster
                .pending_events(true)
                .map(|(node_id, state, iter)| {
                    let pending_events = iter
                        .map(|(id, event)| ClusterNodePendingEvent {
                            id,
                            event: event.to_string(),
                            details: event_details(state, event),
                        })
                        .collect();
                    ClusterNodePendingEvents {
                        node_id,
                        pending_events,
                    }
                })
                .collect()
        })
        .map(Json)
}

async fn cluster_node_events_pending(
    State(state): State<AppState>,
    Path((cluster_id, node_id)): Path<(u16, ClusterNodeId)>,
) -> Result<Json<Vec<ClusterNodePendingEvent>>, (StatusCode, String)> {
    let mut cluster = state.cluster(cluster_id).await?;
    cluster
        .node_pending_events(node_id, true)
        .map(|(state, iter)| {
            iter.map(|(id, event)| ClusterNodePendingEvent {
                id,
                event: event.to_string(),
                details: event_details(state, event),
            })
            .collect()
        })
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

#[derive(Serialize)]
struct ClusterDestroyResponse {
    existed: bool,
}

async fn cluster_destroy(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Json<ClusterDestroyResponse> {
    let existed = state.cluster_destroy(cluster_id).await;
    Json(ClusterDestroyResponse { existed })
}

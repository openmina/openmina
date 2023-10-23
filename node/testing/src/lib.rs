mod exit_with_error;
pub use exit_with_error::exit_with_error;

pub mod service;
use service::PendingEventId;

pub mod node;
use crate::node::NodeTestingConfig;

pub mod cluster;
use cluster::{Cluster, ClusterConfig, ClusterNodeId};

pub mod scenario;
use scenario::{event_details, Scenario, ScenarioId, ScenarioInfo, ScenarioStep};

#[cfg(feature = "scenario-generators")]
pub mod scenarios;

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, Mutex, MutexGuard, OwnedMutexGuard};
use tower_http::cors::CorsLayer;

pub fn server(port: u16) {
    eprintln!("scenarios path: {}", Scenario::PATH);

    let state = AppState::new();

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
        .route("/:cluster_id/run", post(cluster_run))
        .route("/:cluster_id/run/auto", post(cluster_run_auto))
        .route(
            "/:cluster_id/scenarios/reload",
            post(cluster_scenarios_reload),
        )
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

    let app = Router::new()
        .nest("/scenarios", scenarios_router)
        .nest("/clusters", clusters_router)
        .with_state(state)
        .layer(cors);

    tokio::runtime::Handle::current().block_on(async {
        axum::Server::bind(&([0, 0, 0, 0], port).into())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}

pub struct AppStateInner {
    rng: StdRng,
    clusters: BTreeMap<u16, Arc<Mutex<Cluster>>>,
}

impl AppStateInner {
    pub fn new() -> Self {
        Self {
            rng: StdRng::seed_from_u64(0),
            clusters: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct AppState(Arc<Mutex<AppStateInner>>);

impl AppState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(AppStateInner::new())))
    }

    pub async fn lock(&self) -> MutexGuard<'_, AppStateInner> {
        self.0.lock().await
    }

    async fn cluster_mutex(
        &self,
        cluster_id: u16,
    ) -> Result<Arc<Mutex<Cluster>>, (StatusCode, String)> {
        let state = self.lock().await;
        state.clusters.get(&cluster_id).cloned().ok_or_else(|| {
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
        Ok(self.cluster_mutex(cluster_id).await?.lock_owned().await)
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

    pub async fn cluster_destroy(&self, cluster_id: u16) -> bool {
        self.lock().await.clusters.remove(&cluster_id).is_some()
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
    let Json(config) = args.unwrap_or_default();
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
                .pending_events()
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
                .pending_events()
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
        .node_pending_events(node_id)
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

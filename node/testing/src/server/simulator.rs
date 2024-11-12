use std::{sync::Arc, time::Duration};

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::put,
};
use mina_p2p_messages::v2;
use openmina_core::channels::oneshot;
use serde::{Deserialize, Serialize};

use crate::{
    cluster::ClusterConfig,
    scenarios::ClusterRunner,
    simulator::{Simulator, SimulatorConfig},
};

use super::AppState;

pub fn simulations_router() -> axum::Router<AppState> {
    axum::Router::new().route("/", put(simulation_create))
}

#[derive(Deserialize)]
struct SimulationCreateArgs {
    cluster: ClusterConfig,
    simulator: SimulatorConfig,
    #[serde(default)]
    override_genesis_state_timestamp: bool,
}

#[derive(Serialize)]
struct SimulationCreateResponse {
    cluster_id: u16,
}

async fn simulation_create(
    State(state): State<AppState>,
    Json(args): Json<SimulationCreateArgs>,
) -> Result<Json<SimulationCreateResponse>, (StatusCode, String)> {
    async fn setup(
        state: AppState,
        mut args: SimulationCreateArgs,
    ) -> Result<(u16, Simulator), (StatusCode, String)> {
        let (cluster_id, mut cluster) = state.cluster_create_empty(args.cluster).await?;

        let initial_time = redux::Timestamp::global_now();
        if args.override_genesis_state_timestamp {
            Arc::get_mut(&mut args.simulator.genesis)
                .unwrap()
                .override_genesis_state_timestamp(v2::BlockTimeTimeStableV1(
                    (u64::from(initial_time) / 1_000_000).into(),
                ));
        }
        let mut simulator = Simulator::new(initial_time, args.simulator);
        simulator
            .setup(&mut ClusterRunner::new(&mut cluster, |_| {}))
            .await;
        Ok((cluster_id, simulator))
    }
    let (setup_tx, setup_rx) = oneshot::channel();
    let state_clone = state.clone();

    std::thread::spawn(move || {
        let state = state_clone;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let (cluster_id, mut simulator) = match setup(state.clone(), args).await {
                Err(err) => {
                    let _ = setup_tx.send(Err(err));
                    return;
                }
                Ok((cluster_id, simulator)) => {
                    let _ = setup_tx.send(Ok(cluster_id));
                    (cluster_id, simulator)
                }
            };
            let cluster_mutex = match state.cluster_mutex(cluster_id).await {
                Err(_) => return,
                Ok(cluster_mutex) => Arc::downgrade(&cluster_mutex),
            };
            while let Some(cluster_mutex) = cluster_mutex.upgrade() {
                let mut cluster = cluster_mutex.lock().await;
                let mut runner = ClusterRunner::new(&mut *cluster, |_| {});
                let _ =
                    tokio::time::timeout(Duration::from_millis(500), simulator.run(&mut runner))
                        .await;
            }
        });
    });
    let cluster_id = setup_rx.await.unwrap()?;

    Ok(SimulationCreateResponse { cluster_id }).map(Json)
}

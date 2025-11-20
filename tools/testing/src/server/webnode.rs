use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    routing::get,
};
use rand::prelude::*;

use crate::scenarios::ClusterRunner;

use super::AppState;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(webnode_get))
        .route("/lock-random-bp", get(webnode_lock_random_bp))
}

async fn webnode_get(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Result<Redirect, (StatusCode, String)> {
    use base64::{engine::general_purpose::URL_SAFE, Engine as _};
    // make sure cluster exists
    state.cluster_mutex(cluster_id).await?;

    let args = serde_json::json!({
        "network": cluster_id,
    })
    .to_string();
    let args = URL_SAFE.encode(&args);

    Ok(Redirect::temporary(&format!("/?a={args}")))
}

async fn webnode_lock_random_bp(
    State(state): State<AppState>,
    Path(cluster_id): Path<u16>,
) -> Result<Redirect, (StatusCode, String)> {
    use base64::{engine::general_purpose::URL_SAFE, Engine as _};
    let mut state_guard = state.lock().await;
    let state = &mut *state_guard;
    let mut cluster = state.cluster(cluster_id).await?;
    let runner = ClusterRunner::new(&mut cluster, |_| {});
    let locked_keys = state
        .locked_block_producer_keys
        .entry(cluster_id)
        .or_default();

    let (sec_key, _) = runner
        .block_producer_sec_keys(runner.nodes_iter().next().unwrap().0)
        .into_iter()
        .filter(|(key, _)| !locked_keys.contains(&key.public_key()))
        .choose(&mut state.rng)
        .ok_or_else(|| {
            (
                StatusCode::NOT_ACCEPTABLE,
                "no more block producer keys available!".to_owned(),
            )
        })?;
    locked_keys.insert(sec_key.public_key());

    let args = serde_json::json!({
        "network": cluster_id,
        "block_producer": {
            "sec_key": sec_key.to_string(),
            "pub_key": sec_key.public_key().to_string(),
        },
    })
    .to_string();
    let args = URL_SAFE.encode(&args);

    Ok(Redirect::temporary(&format!("/?a={args}")))
}

use reqwest::StatusCode;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{evaluator::epoch::{EpochSlots, EpochStorage}, node::{self, NodeData}, storage::db_sled::Database, NodeStatus, StakingToolError};

pub async fn get_genesis_timestamp() -> Result<impl warp::Reply, warp::reject::Rejection> {
    // TODO(adonagy): we need this only once, no need to query the node every time...
    // match node::get_genesis_timestmap().await {
    //     Ok(timestamp) => {
    //         let datetime = OffsetDateTime::parse(&timestamp, &Rfc3339).unwrap();
    //         let unix_timestamp = datetime.unix_timestamp();
    //         Ok(warp::reply::with_status(
    //             warp::reply::json(&unix_timestamp.to_string()),
    //             StatusCode::OK,
    //         ))
    //     }
    //     // TODO(adonagy)
    //     Err(_) => Err(warp::reject()),
    // }

    Ok(warp::reply())
}

pub async fn get_node_status(node_status: NodeStatus) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let node_status: NodeData = node_status.read().await.clone();

    Ok(warp::reply::with_status(
        warp::reply::json(&node_status),
        StatusCode::OK,
    ))
}

pub async fn get_current_slot(node_status: NodeStatus) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let current_slot = node_status.read().await.current_slot();

    Ok(warp::reply::with_status(
        warp::reply::json(&current_slot),
        StatusCode::OK,
    ))
}

pub async fn get_latest_epoch_data(
    storage: Database,
    node_status: NodeStatus,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let node_status = node_status.read().await;

    let current_epoch = node_status.best_tip().epoch();

    match storage.get_slots_for_epoch(current_epoch) {
        Ok(latest) => Ok(warp::reply::with_status(
            warp::reply::json(&latest),
            StatusCode::OK,
        )),
        // TODO(adonagy)
        _ => Err(warp::reject()),
    }
}

pub async fn get_latest_epoch_data_summary(
    storage: Database,
    node_status: NodeStatus,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let node_status = node_status.read().await;

    let current_epoch = node_status.best_tip().epoch();

    match storage.get_slots_for_epoch(current_epoch) {
        Ok(latest) => {

            let summary = EpochSlots::new(current_epoch, latest).summary();

            Ok(warp::reply::with_status(
                warp::reply::json(&summary),
                StatusCode::OK,
            ))
        },
        // TODO(adonagy)
        _ => Err(warp::reject()),
    }
}

use reqwest::StatusCode;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{evaluator::epoch::EpochStorage, node, StakingToolError, storage::db_sled::Database};

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

pub async fn get_latest_epoch_data(
    storage: Database,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    match storage.get_seed(4) {
        Ok(Some(latest)) => Ok(warp::reply::with_status(
            warp::reply::json(&vec![latest]),
            StatusCode::OK,
        )),
        // TODO(adonagy)
        _ => Err(warp::reject()),
    }
}

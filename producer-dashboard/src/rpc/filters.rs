use warp::Filter;

use crate::storage::db_sled::Database;

use super::handlers::{get_genesis_timestamp, get_latest_epoch_data};

pub fn filters(
    storage: Database,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_method("GET");

    genesis_timestamp()
        .or(latest_epoch_data(storage))
        .with(cors)
}

fn genesis_timestamp() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("genesis_timestamp")
        .and(warp::get())
        .and_then(get_genesis_timestamp)
}

fn latest_epoch_data(
    storage: Database,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("epoch" / "latest")
        .and(warp::get())
        .and(with_storage(storage))
        .and_then(get_latest_epoch_data)
}

fn with_storage(
    storage: Database,
) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

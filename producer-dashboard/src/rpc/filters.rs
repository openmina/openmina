use warp::Filter;

use crate::EpochStorage;

use super::handlers::get_genesis_timestamp;

pub fn filters(storage: EpochStorage) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_method("GET");

    genesis_timestamp().with(cors)
}

fn genesis_timestamp() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("genesis_timestamp")
        .and(warp::get())
        .and_then(get_genesis_timestamp)
}
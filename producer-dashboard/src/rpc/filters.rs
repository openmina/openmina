use warp::Filter;

use crate::{storage::db_sled::Database, NodeStatus};

use super::{
    handlers::{
        get_current_slot, get_epoch_data, get_epoch_data_summary, get_genesis_timestamp,
        get_latest_epoch_data, get_latest_epoch_data_summary, get_node_status,
    },
    PaginationParams,
};

pub fn filters(
    storage: Database,
    node_status: NodeStatus,
    producer_pk: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_method("GET");

    genesis_timestamp()
        .or(latest_epoch_data(storage.clone(), node_status.clone()))
        .or(node(node_status.clone()))
        .or(current_slot(node_status.clone()))
        .or(epoch_summary(storage.clone(), producer_pk.clone()))
        .or(epoch_data(storage.clone()))
        .or(latest_epoch_summary(storage, node_status, producer_pk))
        .with(cors)
}

fn genesis_timestamp() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("genesis_timestamp")
        .and(warp::get())
        .and_then(get_genesis_timestamp)
}

fn node(
    node_status: NodeStatus,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("node")
        .and(warp::get())
        .and(with_node_status(node_status))
        .and_then(get_node_status)
}

fn current_slot(
    node_status: NodeStatus,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("node" / "current_slot")
        .and(warp::get())
        .and(with_node_status(node_status))
        .and_then(get_current_slot)
}

fn latest_epoch_data(
    storage: Database,
    node_status: NodeStatus,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("epoch" / "latest")
        .and(warp::get())
        .and(with_storage(storage))
        .and(with_node_status(node_status))
        .and_then(get_latest_epoch_data)
}

fn epoch_data(
    storage: Database,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("epoch" / u32)
        .and(warp::get())
        .and(with_storage(storage))
        .and_then(get_epoch_data)
}

fn latest_epoch_summary(
    storage: Database,
    node_status: NodeStatus,
    producer_pk: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("epoch" / "summary" / "latest")
        .and(warp::get())
        .and(with_storage(storage))
        .and(with_node_status(node_status))
        .and(with_producer_pk(producer_pk))
        .and_then(get_latest_epoch_data_summary)
}

fn epoch_summary(
    storage: Database,
    producer_pk: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("epoch" / "summary" / u32)
        .and(warp::get())
        .and(warp::query::<PaginationParams>())
        .and(with_storage(storage))
        .and(with_producer_pk(producer_pk))
        .and_then(get_epoch_data_summary)
}

fn with_storage(
    storage: Database,
) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

fn with_node_status(
    node_status: NodeStatus,
) -> impl Filter<Extract = (NodeStatus,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || node_status.clone())
}

fn with_producer_pk(
    producer_pk: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || producer_pk.clone())
}

use reqwest::StatusCode;

use crate::{
    evaluator::epoch::EpochSlots,
    node::{epoch_ledgers::Balances, NodeData},
    storage::db_sled::Database,
    NodeStatus,
};

use super::PaginationParams;

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

pub async fn get_node_status(
    node_status: NodeStatus,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let node_status: NodeData = node_status.read().await.clone();

    Ok(warp::reply::with_status(
        warp::reply::json(&node_status),
        StatusCode::OK,
    ))
}

pub async fn get_current_slot(
    node_status: NodeStatus,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
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

    let current_epoch = node_status.best_tip().unwrap().epoch();

    match storage.get_slots_for_epoch(current_epoch) {
        Ok(latest) => Ok(warp::reply::with_status(
            warp::reply::json(&latest),
            StatusCode::OK,
        )),
        // TODO(adonagy)
        _ => Err(warp::reject()),
    }
}

pub async fn get_epoch_data(
    epoch: u32,
    storage: Database,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    match storage.get_slots_for_epoch(epoch) {
        Ok(slots) => Ok(warp::reply::with_status(
            warp::reply::json(&slots),
            StatusCode::OK,
        )),
        // TODO(adonagy)
        _ => Err(warp::reject()),
    }
}

pub async fn get_latest_epoch_data_summary(
    storage: Database,
    node_status: NodeStatus,
    producer_pk: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let node_status: tokio::sync::RwLockReadGuard<'_, NodeData> = node_status.read().await;

    let current_epoch = node_status.best_tip().unwrap().epoch();

    let balances = storage
        .get_ledger(current_epoch)
        .unwrap()
        .unwrap()
        .producer_balances(&producer_pk);

    match storage.get_slots_for_epoch(current_epoch) {
        Ok(latest) => {
            let summary = EpochSlots::new(latest).merged_summary(current_epoch, balances);

            Ok(warp::reply::with_status(
                warp::reply::json(&summary),
                StatusCode::OK,
            ))
        }
        // TODO(adonagy)
        _ => Err(warp::reject()),
    }
}

pub async fn get_epoch_data_summary(
    requested_epoch: u32,
    pagination: PaginationParams,
    storage: Database,
    producer_pk: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let range = (0..=requested_epoch).rev();
    let limit = pagination.limit.unwrap_or(1);

    let mut res = Vec::new();

    for epoch in range.take(limit) {
        println!("{epoch}");
        match storage.get_slots_for_epoch(epoch) {
            Ok(slots) => {
                let balances = match storage.get_ledger(epoch) {
                    Ok(Some(ledger)) => ledger.producer_balances(&producer_pk),
                    _ => Balances::default(),
                };
                res.push(EpochSlots::new(slots).merged_summary(epoch, balances))
            }
            _ => continue,
        }
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&res),
        StatusCode::OK,
    ))
}

pub async fn get_all_time_summary(
    storage: Database,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    match storage.get_all_slots() {
        Ok(slots) => {
            let res = EpochSlots::new(slots).slot_summary().0;
            Ok(warp::reply::with_status(
                warp::reply::json(&res),
                StatusCode::OK,
            ))
        }
        _ => Err(warp::reject()),
    }
}

pub mod filters;
pub mod handlers;

use serde::Deserialize;
use tokio::task::JoinHandle;

use crate::{storage::db_sled::Database, NodeStatus};

pub fn spawn_rpc_server(
    port: u16,
    db: Database,
    node_status: NodeStatus,
    producer_pk: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let api = filters::filters(db.clone(), node_status.clone(), producer_pk);

        warp::serve(api).run(([0, 0, 0, 0], port)).await;
    })
}

#[derive(Deserialize)]
pub struct PaginationParams {
    limit: Option<usize>,
}

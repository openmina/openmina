pub mod filters;
pub mod handlers;

use tokio::task::JoinHandle;

use crate::{evaluator::epoch::EpochStorage, storage::db_sled::Database, NodeStatus};

pub fn spawn_rpc_server(port: u16, db: Database, node_status: NodeStatus) -> JoinHandle<()> {
    tokio::spawn(async move {
        let api = filters::filters(db.clone(), node_status.clone());

        warp::serve(api).run(([0, 0, 0, 0], port)).await;
    })
}

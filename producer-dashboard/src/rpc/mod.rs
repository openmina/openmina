pub mod filters;
pub mod handlers;

use tokio::task::JoinHandle;

use crate::{evaluator::epoch::EpochStorage, storage::db_sled::Database};

pub fn spawn_rpc_server(port: u16, db: Database) -> JoinHandle<()> {
    tokio::spawn(async move {
        let api = filters::filters(db.clone());

        warp::serve(api).run(([0, 0, 0, 0], port)).await;
    })
}

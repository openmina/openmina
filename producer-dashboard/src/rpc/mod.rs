pub mod filters;
pub mod handlers;

use tokio::task::JoinHandle;

use crate::epoch::EpochStorage;

pub fn spawn_rpc_server(port: u16, storage: EpochStorage) -> JoinHandle<()> {
    tokio::spawn(async move {
        let api = filters::filters(storage.clone());

        warp::serve(api).run(([0, 0, 0, 0], port)).await;
    })
}

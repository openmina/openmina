mod builder;
pub use builder::*;

use openmina_node_common::p2p::TaskSpawner;
pub use openmina_node_common::NodeService;

#[derive(Clone)]
pub struct P2pTaskSpawner {}

impl TaskSpawner for P2pTaskSpawner {
    fn spawn_main<F>(&self, name: &str, fut: F)
    where
        F: 'static + Send + std::future::Future,
    {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        std::thread::Builder::new()
            .name(format!("openmina_p2p_{name}"))
            .spawn(move || {
                let local_set = tokio::task::LocalSet::new();
                local_set.block_on(&runtime, fut);
            })
            .unwrap();
    }
}

mod builder;
pub use builder::*;

pub type Node = openmina_node_common::Node<crate::NodeService>;

use ::node::core::thread;
use std::future::Future;

#[derive(Clone)]
pub struct P2pTaskSpawner {}

impl node::p2p::service_impl::TaskSpawner for P2pTaskSpawner {
    fn spawn_main<F>(&self, _name: &str, fut: F)
    where
        F: 'static + Send + std::future::Future<Output = ()>,
    {
        wasm_bindgen_futures::spawn_local(fut);
    }
}

/// Created in the main thread, passed to the worker thread.
/// Main task which is supposed to be executed in the p2p thread, is
/// sent via channel to the main thread. This is needed for p2p as
/// webrtc js api doesn't exist in the web worker.
#[derive(Clone)]
pub struct P2pTaskRemoteSpawner {}

impl node::p2p::service_impl::TaskSpawner for P2pTaskRemoteSpawner {
    fn spawn_main<F>(&self, _name: &str, fut: F)
    where
        F: 'static + Send + Future<Output = ()>,
    {
        thread::start_task_in_main_thread(fut);
    }
}

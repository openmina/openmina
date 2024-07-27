mod builder;
pub use builder::*;

pub type Node = openmina_node_common::Node<crate::NodeService>;

use ::node::core::{channels::mpsc, thread};
use std::future::Future;
use std::pin::Pin;

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

pub type P2pTaskRemoteSpawnerMainTask = Pin<Box<dyn 'static + Send + Future<Output = ()>>>;

/// Created in the main thread, passed to the worker thread.
/// Main task which is supposed to be executed in the p2p thread, is
/// sent via channel to the main thread. This is needed for p2p as
/// webrtc js api doesn't exist in the web worker.
#[derive(Clone)]
pub struct P2pTaskRemoteSpawner {
    task_sender: mpsc::Sender<P2pTaskRemoteSpawnerMainTask>,
}

impl P2pTaskRemoteSpawner {
    /// Must be called in the main thread.
    pub fn create() -> Self {
        assert!(
            !thread::is_web_worker_thread(),
            "Must be called in the main thread!"
        );

        let (task_sender, mut task_receiver) = mpsc::channel(1);
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(task) = task_receiver.recv().await {
                wasm_bindgen_futures::spawn_local(task);
            }
        });

        Self { task_sender }
    }
}

impl node::p2p::service_impl::TaskSpawner for P2pTaskRemoteSpawner {
    fn spawn_main<F>(&self, _name: &str, fut: F)
    where
        F: 'static + Send + Future<Output = ()>,
    {
        self.task_sender
            .try_send(Box::pin(fut))
            .expect("p2p main task already running?");
    }
}

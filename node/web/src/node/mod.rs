mod builder;
pub use builder::*;

pub type Node = openmina_node_common::Node<crate::NodeService>;

#[derive(Clone)]
pub struct P2pTaskSpawner {}

impl node::p2p::service_impl::TaskSpawner for P2pTaskSpawner {
    fn spawn_main<F>(&self, _name: &str, fut: F)
    where
        F: 'static + Send + std::future::Future<Output = ()>,
    {
        todo!()
    }
}

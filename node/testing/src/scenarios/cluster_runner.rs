use std::time::Duration;

use node::{event_source::Event, State};

use crate::{
    cluster::{Cluster, ClusterNodeId, ClusterOcamlNodeId},
    network_debugger::Debugger,
    node::{Node, OcamlNode, OcamlNodeTestingConfig, RustNodeTestingConfig},
    scenario::ScenarioStep,
    service::PendingEventId,
};

pub struct ClusterRunner<'a> {
    cluster: &'a mut Cluster,
    add_step: Box<dyn 'a + FnMut(&ScenarioStep)>,
}

impl<'a> ClusterRunner<'a> {
    pub fn new<F>(cluster: &'a mut Cluster, add_step: F) -> Self
    where
        F: 'a + FnMut(&ScenarioStep),
    {
        Self {
            cluster,
            add_step: Box::new(add_step),
        }
    }

    pub fn node(&self, node_id: ClusterNodeId) -> Option<&Node> {
        self.cluster.node(node_id)
    }

    pub fn ocaml_node(&self, node_id: ClusterOcamlNodeId) -> Option<&OcamlNode> {
        self.cluster.ocaml_node(node_id)
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = (ClusterNodeId, &Node)> {
        self.cluster.nodes_iter()
    }

    pub fn add_rust_node(&mut self, testing_config: RustNodeTestingConfig) -> ClusterNodeId {
        self.cluster.add_rust_node(testing_config)
    }

    pub fn add_ocaml_node(&mut self, testing_config: OcamlNodeTestingConfig) -> ClusterOcamlNodeId {
        self.cluster.add_ocaml_node(testing_config)
    }

    pub async fn exec_step(&mut self, step: ScenarioStep) -> anyhow::Result<bool> {
        (self.add_step)(&step);
        self.cluster.exec_step(step).await
    }

    pub fn pending_events(
        &mut self,
    ) -> impl Iterator<
        Item = (
            ClusterNodeId,
            &State,
            impl Iterator<Item = (PendingEventId, &Event)>,
        ),
    > {
        self.cluster.pending_events()
    }

    pub fn node_pending_events(
        &mut self,
        node_id: ClusterNodeId,
    ) -> anyhow::Result<(&State, impl Iterator<Item = (PendingEventId, &Event)>)> {
        self.cluster.node_pending_events(node_id)
    }

    pub async fn wait_for_pending_events(&mut self) {
        self.cluster.wait_for_pending_events().await
    }

    pub async fn wait_for_pending_events_with_timeout(&mut self, timeout: Duration) -> bool {
        self.cluster
            .wait_for_pending_events_with_timeout(timeout)
            .await
    }

    pub fn debugger(&self) -> Option<&Debugger> {
        self.cluster.debugger()
    }
}

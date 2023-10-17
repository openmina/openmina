use std::time::Duration;

use node::{event_source::Event, State};

use crate::{
    cluster::{Cluster, ClusterNodeId},
    node::{Node, NodeTestingConfig, RustNodeTestingConfig},
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

    pub fn cluster(&self) -> &Cluster {
        &self.cluster
    }

    pub fn node(&self, node_id: ClusterNodeId) -> Option<&Node> {
        self.cluster.node(node_id)
    }

    pub fn add_node(&mut self, testing_config: NodeTestingConfig) -> ClusterNodeId {
        self.cluster.add_node(testing_config)
    }

    pub fn add_rust_node(&mut self, testing_config: RustNodeTestingConfig) -> ClusterNodeId {
        self.cluster.add_rust_node(testing_config)
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
}

use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use node::{event_source::Event, ActionWithMeta, State};

use crate::{
    cluster::{Cluster, ClusterNodeId, ClusterOcamlNodeId},
    network_debugger::Debugger,
    node::{Node, OcamlNode, OcamlNodeTestingConfig, RustNodeTestingConfig},
    scenario::ScenarioStep,
    service::{DynEffects, NodeTestingService, PendingEventId},
};

pub struct ClusterRunner<'a> {
    cluster: &'a mut Cluster,
    add_step: Box<dyn 'a + FnMut(&ScenarioStep)>,
}

#[derive(Debug, Clone, Copy)]
pub enum RunDecision {
    /// Skip current event without executing it and stop the loop.
    Stop,
    /// Execute current event and stop the loop.
    StopExec,
    /// Skip current event without executing it.
    Skip,
    /// Execute current event and continue.
    ContinueExec,
}

pub struct DynEffectsData<T>(Arc<Mutex<T>>);

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

    fn node_mut(&mut self, node_id: ClusterNodeId) -> Option<&mut Node> {
        self.cluster.node_mut(node_id)
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

    // TODO(binier): better names for `handle_event`, `
    /// Execute cluster in the infinite loop, until `handle_event`,
    /// `handle_action` or `timeout` causes it to end.
    ///
    /// - `timeout` represents timeout for the whole function. It must
    ///   finish before timeout is triggered. For it to finish either
    ///   `handle_event` or `handle_action` must cause infinite loop to end.
    /// - `handle_event` function control execution of events based on
    ///   decision that it will return. It might exec event, skip it,
    ///   and/or stop this infinite loop all together.
    /// - `handle_action` function can react to actions triggered in the
    ///   cluster in order to stop the loop.
    pub async fn run<EH, AH>(
        &mut self,
        timeout: Duration,
        mut handle_event: EH,
        mut exit_if_action: AH,
    ) -> anyhow::Result<()>
    where
        EH: FnMut(ClusterNodeId, &State, &Event) -> RunDecision,
        AH: 'static
            + Send
            + FnMut(ClusterNodeId, &State, &NodeTestingService, &ActionWithMeta) -> bool,
    {
        #[derive(Default)]
        struct Data {
            exit: bool,
            node_id: Option<ClusterNodeId>,
        }

        let dyn_effects_data = DynEffectsData::new(Data::default());
        let dyn_effects_data_clone = dyn_effects_data.clone();
        let mut dyn_effects = Box::new(
            move |state: &State, service: &NodeTestingService, action: &ActionWithMeta| {
                let mut data = dyn_effects_data_clone.inner();
                if let Some(node_id) = data.node_id {
                    data.exit = exit_if_action(node_id, state, service, action);
                }
            },
        ) as DynEffects;
        tokio::time::timeout(timeout, async move {
            loop {
                let event_to_take_action_on = self
                    .pending_events()
                    .flat_map(|(node_id, state, events)| {
                        events.map(move |event| (node_id, state, event))
                    })
                    .map(|(node_id, state, (_, event))| {
                        let decision = handle_event(node_id, state, event);
                        (node_id, state, event, decision)
                    })
                    .find(|(_, _, _, decision)| decision.stop() || decision.exec());

                if let Some((node_id, _, event, decision)) = event_to_take_action_on {
                    dyn_effects_data.inner().node_id = Some(node_id);
                    if decision.exec() {
                        let event = event.to_string();
                        self.node_mut(node_id).unwrap().set_dyn_effects(dyn_effects);
                        self.exec_step(ScenarioStep::Event { node_id, event })
                            .await
                            .unwrap();
                        dyn_effects = self
                            .node_mut(node_id)
                            .unwrap()
                            .remove_dyn_effects()
                            .unwrap();

                        if dyn_effects_data.inner().exit || decision.stop() {
                            return;
                        }
                        continue;
                    }

                    if decision.stop() {
                        return;
                    }
                }

                let all_nodes = self.nodes_iter().map(|(id, _)| id).collect::<Vec<_>>();

                for node_id in all_nodes {
                    self.cluster
                        .exec_step(ScenarioStep::CheckTimeouts { node_id })
                        .await
                        .unwrap();
                }

                self.wait_for_pending_events().await;
            }
        })
        .await
        .map_err(|_| {
            anyhow::anyhow!(
                "timeout({} ms) has elapsed during `run`",
                timeout.as_millis()
            )
        })
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

impl RunDecision {
    pub fn stop(self) -> bool {
        match self {
            Self::Stop => true,
            Self::StopExec => true,
            Self::Skip => false,
            Self::ContinueExec => false,
        }
    }

    pub fn exec(self) -> bool {
        match self {
            Self::Stop => false,
            Self::StopExec => true,
            Self::Skip => false,
            Self::ContinueExec => true,
        }
    }
}

impl<T> DynEffectsData<T> {
    pub fn new(data: T) -> Self {
        Self(Arc::new(Mutex::new(data)))
    }

    pub fn inner(&self) -> MutexGuard<'_, T> {
        self.0
            .try_lock()
            .expect("DynEffectsData is never expected to be accessed from multiple threads")
    }
}

impl<T> Clone for DynEffectsData<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

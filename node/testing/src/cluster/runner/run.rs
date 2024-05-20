use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use node::{event_source::Event, ActionWithMeta, State};
use rand::Rng;

use crate::{
    cluster::ClusterNodeId,
    scenario::ScenarioStep,
    service::{DynEffects, NodeTestingService},
};

pub struct RunCfg<
    EH: FnMut(ClusterNodeId, &State, &Event) -> RunDecision,
    AH: 'static + Send + FnMut(ClusterNodeId, &State, &NodeTestingService, &ActionWithMeta) -> bool,
> {
    timeout: Duration,
    handle_event: EH,
    exit_if_action: AH,
    advance_time: Option<std::ops::RangeInclusive<u64>>,
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

impl<'a> super::ClusterRunner<'a> {
    /// Execute cluster in the infinite loop, based on conditions specified
    /// in the `RunCfg`.
    pub async fn run<EH, AH>(
        &mut self,
        RunCfg {
            timeout,
            advance_time,
            mut handle_event,
            mut exit_if_action,
        }: RunCfg<EH, AH>,
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
                    data.exit |= exit_if_action(node_id, state, service, action);
                }
            },
        ) as DynEffects;
        tokio::time::timeout(timeout, async move {
            while !dyn_effects_data.inner().exit {
                let event_to_take_action_on = self
                    .pending_events(true)
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
                        dyn_effects = self
                            .exec_step_with_dyn_effects(
                                dyn_effects,
                                node_id,
                                ScenarioStep::Event { node_id, event },
                            )
                            .await;

                        if decision.stop() {
                            return;
                        }
                        continue;
                    }

                    if decision.stop() {
                        return;
                    }
                }

                if let Some(time) = advance_time.as_ref() {
                    let (start, end) = time.clone().into_inner();
                    let (start, end) = (start * 1_000_000, end * 1_000_000);
                    let by_nanos = self.rng.gen_range(start..end);
                    self.exec_step(ScenarioStep::AdvanceTime { by_nanos })
                        .await
                        .unwrap();
                }

                let all_nodes = self.nodes_iter().map(|(id, _)| id).collect::<Vec<_>>();
                for node_id in all_nodes {
                    dyn_effects_data.inner().node_id = Some(node_id);
                    dyn_effects = self
                        .exec_step_with_dyn_effects(
                            dyn_effects,
                            node_id,
                            ScenarioStep::CheckTimeouts { node_id },
                        )
                        .await;
                    if dyn_effects_data.inner().exit {
                        return;
                    }
                }

                if advance_time.is_some() {
                    self.wait_for_pending_events_with_timeout(Duration::from_millis(100))
                        .await;
                } else {
                    self.wait_for_pending_events().await;
                }
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
}

impl Default
    for RunCfg<
        fn(ClusterNodeId, &State, &Event) -> RunDecision,
        fn(ClusterNodeId, &State, &NodeTestingService, &ActionWithMeta) -> bool,
    >
{
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            advance_time: None,
            handle_event: |_, _, _| RunDecision::ContinueExec,
            exit_if_action: |_, _, _, _| false,
        }
    }
}

impl<EH, AH> RunCfg<EH, AH>
where
    EH: FnMut(ClusterNodeId, &State, &Event) -> RunDecision,
    AH: 'static + Send + FnMut(ClusterNodeId, &State, &NodeTestingService, &ActionWithMeta) -> bool,
{
    /// Set `timeout` for the whole `run` function.
    ///
    /// `run` function will time out, unless `event_handler` or `action_handler`
    /// causes it to end before the timeout duration elapses.
    ///
    /// Default: 60s
    pub fn timeout(mut self, dur: Duration) -> Self {
        self.timeout = dur;
        self
    }

    /// Set the range of time in milliseconds, with which time will be
    /// advanced during `run` function execution.
    ///
    /// By default `run` function won't advance time.
    pub fn advance_time(mut self, range: std::ops::RangeInclusive<u64>) -> Self {
        self.advance_time = Some(range);
        self
    }

    /// Set function control execution of events based on decision that
    /// it will return. It might exec event, skip it, and/or end the
    /// execution of the `run` function.
    pub fn event_handler<NewEh>(self, handler: NewEh) -> RunCfg<NewEh, AH>
    where
        NewEh: FnMut(ClusterNodeId, &State, &Event) -> RunDecision,
    {
        RunCfg {
            timeout: self.timeout,
            advance_time: self.advance_time,
            handle_event: handler,
            exit_if_action: self.exit_if_action,
        }
    }

    /// Set function using which `run` function can be stopped based on
    /// the passed predicate. It can also be used to gather some data
    /// based on actions to be used in tests.
    pub fn action_handler<NewAH>(self, handler: NewAH) -> RunCfg<EH, NewAH>
    where
        NewAH: 'static
            + Send
            + FnMut(ClusterNodeId, &State, &NodeTestingService, &ActionWithMeta) -> bool,
    {
        RunCfg {
            timeout: self.timeout,
            advance_time: self.advance_time,
            handle_event: self.handle_event,
            exit_if_action: handler,
        }
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

use redux::EnablingCondition;
use serde::{Deserialize, Serialize};
use shared::snark_job_id::SnarkJobId;

use crate::State;

use super::{ExternalSnarkWorkerError, ExternalSnarkWorkerState, ExternalSnarkWorkerWorkError};

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From)]
pub enum ExternalSnarkWorkerAction {
    Start(ExternalSnarkWorkerStartAction),
    Started(ExternalSnarkWorkerStartedAction),
    Kill(ExternalSnarkWorkerKillAction),
    Killed(ExternalSnarkWorkerKilledAction),

    SubmitWork(ExternalSnarkWorkerSubmitWorkAction),
    WorkResult(ExternalSnarkWorkerWorkResultAction),
    WorkError(ExternalSnarkWorkerWorkErrorAction),

    CancelWork(ExternalSnarkWorkerCancelWorkAction),
    WorkCancelled(ExternalSnarkWorkerWorkCancelledAction),

    PruneWork(ExternalSnarkWorkerPruneWorkAction),

    Error(ExternalSnarkWorkerErrorAction),
}

pub type ExternalSnarkWorkerActionWithMeta = redux::ActionWithMeta<ExternalSnarkWorkerAction>;
pub type ExternalSnarkWorkerActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a ExternalSnarkWorkerAction>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerStartAction {}

impl EnablingCondition<State> for ExternalSnarkWorkerStartAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        matches!(state.external_snark_worker, ExternalSnarkWorkerState::None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerStartedAction {}

impl EnablingCondition<State> for ExternalSnarkWorkerStartedAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        matches!(
            state.external_snark_worker,
            ExternalSnarkWorkerState::Starting
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerKillAction {}

impl EnablingCondition<State> for ExternalSnarkWorkerKillAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        match &state.external_snark_worker {
            ExternalSnarkWorkerState::None | ExternalSnarkWorkerState::Killing => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerKilledAction {}

impl EnablingCondition<State> for ExternalSnarkWorkerKilledAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        matches!(
            state.external_snark_worker,
            ExternalSnarkWorkerState::Killing
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerErrorAction {
    pub error: ExternalSnarkWorkerError,
}

impl EnablingCondition<State> for ExternalSnarkWorkerErrorAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerSubmitWorkAction {
    pub job_id: SnarkJobId,
}

impl EnablingCondition<State> for ExternalSnarkWorkerSubmitWorkAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        state.external_snark_worker.is_idle()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerCancelWorkAction {}

impl EnablingCondition<State> for ExternalSnarkWorkerCancelWorkAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        matches!(
            state.external_snark_worker,
            ExternalSnarkWorkerState::Working(_)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerWorkCancelledAction {}

impl EnablingCondition<State> for ExternalSnarkWorkerWorkCancelledAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        matches!(
            state.external_snark_worker,
            ExternalSnarkWorkerState::Cancelling(_)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerWorkResultAction {
    pub result: super::SnarkWorkResult,
}

impl EnablingCondition<State> for ExternalSnarkWorkerWorkResultAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        matches!(
            state.external_snark_worker,
            ExternalSnarkWorkerState::Working(_)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerWorkErrorAction {
    pub error: ExternalSnarkWorkerWorkError,
}

impl EnablingCondition<State> for ExternalSnarkWorkerWorkErrorAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        matches!(
            state.external_snark_worker,
            ExternalSnarkWorkerState::Working(_)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkerPruneWorkAction {}

impl EnablingCondition<State> for ExternalSnarkWorkerPruneWorkAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &State) -> bool {
        matches!(
            state.external_snark_worker,
            ExternalSnarkWorkerState::WorkReady(..)
                | ExternalSnarkWorkerState::WorkError(..)
                | ExternalSnarkWorkerState::Cancelled(..)
        )
    }
}

macro_rules! impl_into_global_action {
    ($($a:ty),* $(,)?) => {
        $(
            impl From<$a> for crate::Action {
                fn from(value: $a) -> Self {
                    Self::ExternalSnarkWorker(value.into())
                }
            }
        )*
    };
}

impl_into_global_action!(
    ExternalSnarkWorkerStartAction,
    ExternalSnarkWorkerStartedAction,
    ExternalSnarkWorkerKillAction,
    ExternalSnarkWorkerKilledAction,
    ExternalSnarkWorkerErrorAction,
    ExternalSnarkWorkerSubmitWorkAction,
    ExternalSnarkWorkerWorkResultAction,
    ExternalSnarkWorkerCancelWorkAction,
    ExternalSnarkWorkerWorkCancelledAction,
    ExternalSnarkWorkerPruneWorkAction,
    ExternalSnarkWorkerWorkErrorAction,
);

use std::time::Duration;

use openmina_core::snark::SnarkJobId;
use redux::{EnablingCondition, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{snark_pool::JobSummary, State};

use super::{
    ExternalSnarkWorkerError, ExternalSnarkWorkerState, ExternalSnarkWorkerWorkError,
    SnarkWorkResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalSnarkWorkerAction {
    Start,
    Started,
    StartTimeout {
        now: Timestamp,
    },
    Kill,
    Killed,

    SubmitWork {
        job_id: SnarkJobId,
        summary: JobSummary,
    },
    WorkResult {
        result: SnarkWorkResult,
    },
    WorkError {
        error: ExternalSnarkWorkerWorkError,
    },
    WorkTimeout {
        now: Timestamp,
    },

    CancelWork,
    WorkCancelled,

    PruneWork,

    Error {
        error: ExternalSnarkWorkerError,
        permanent: bool,
    },
}

pub type ExternalSnarkWorkerActionWithMeta = redux::ActionWithMeta<ExternalSnarkWorkerAction>;
pub type ExternalSnarkWorkerActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a ExternalSnarkWorkerAction>;

impl EnablingCondition<State> for ExternalSnarkWorkerAction {
    fn is_enabled(&self, state: &State, _time: redux::Timestamp) -> bool {
        match self {
            ExternalSnarkWorkerAction::Start => {
                state.config.snarker.is_some()
                    && matches!(
                        state.external_snark_worker.0.state,
                        ExternalSnarkWorkerState::None
                    )
            }
            ExternalSnarkWorkerAction::Started => {
                matches!(
                    state.external_snark_worker.0.state,
                    ExternalSnarkWorkerState::Starting
                )
            }
            ExternalSnarkWorkerAction::StartTimeout { now } => {
                const TIMEOUT: Duration = Duration::from_secs(120);
                matches!(
                    state.external_snark_worker.0.state,
                    ExternalSnarkWorkerState::Starting
                ) && now
                    .checked_sub(state.external_snark_worker.0.timestamp)
                    .map_or(false, |d| d > TIMEOUT)
            }
            ExternalSnarkWorkerAction::Kill => !matches!(
                state.external_snark_worker.0.state,
                ExternalSnarkWorkerState::Error(_, false)
                    | ExternalSnarkWorkerState::None
                    | ExternalSnarkWorkerState::Killing
            ),
            ExternalSnarkWorkerAction::Killed => {
                matches!(
                    state.external_snark_worker.0.state,
                    ExternalSnarkWorkerState::Killing
                )
            }
            ExternalSnarkWorkerAction::SubmitWork { .. } => state.external_snark_worker.is_idle(),
            ExternalSnarkWorkerAction::WorkResult { .. } => {
                matches!(
                    state.external_snark_worker.0.state,
                    ExternalSnarkWorkerState::Working(..)
                )
            }
            ExternalSnarkWorkerAction::WorkError { .. } => {
                matches!(
                    state.external_snark_worker.0.state,
                    ExternalSnarkWorkerState::Working(..)
                )
            }
            ExternalSnarkWorkerAction::WorkTimeout { now } => {
                if let ExternalSnarkWorkerState::Working(_, summary) =
                    &state.external_snark_worker.0.state
                {
                    now.checked_sub(state.external_snark_worker.0.timestamp)
                        .map_or(false, |d| d > summary.estimated_duration())
                } else {
                    false
                }
            }
            ExternalSnarkWorkerAction::CancelWork => {
                matches!(
                    state.external_snark_worker.0.state,
                    ExternalSnarkWorkerState::Working(..)
                )
            }
            ExternalSnarkWorkerAction::WorkCancelled => {
                matches!(
                    state.external_snark_worker.0.state,
                    ExternalSnarkWorkerState::Cancelling(_)
                )
            }
            ExternalSnarkWorkerAction::PruneWork => {
                matches!(
                    state.external_snark_worker.0.state,
                    ExternalSnarkWorkerState::WorkReady(..)
                        | ExternalSnarkWorkerState::WorkError(..)
                        | ExternalSnarkWorkerState::Cancelled(..)
                )
            }
            ExternalSnarkWorkerAction::Error { .. } => true,
        }
    }
}

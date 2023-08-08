use std::ffi::OsStr;

use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use serde::{Deserialize, Serialize};

use super::{
    ExternalSnarkWorkerError, ExternalSnarkWorkerWorkError, SnarkWorkResult, SnarkWorkSpec,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExternalSnarkWorkerEvent {
    Started,
    Killed,
    WorkResult(SnarkWorkResult),
    WorkError(ExternalSnarkWorkerWorkError),
    WorkCancelled,
    Error(ExternalSnarkWorkerError),
}

pub trait ExternalSnarkWorkerService {
    /// Starts external process.
    fn start<P: AsRef<OsStr>>(
        &mut self,
        path: P,
        public_key: NonZeroCurvePoint,
        fee: CurrencyFeeStableV1,
    ) -> Result<(), ExternalSnarkWorkerError>;

    /// Submits snark work
    fn submit(&mut self, spec: SnarkWorkSpec) -> Result<(), ExternalSnarkWorkerError>;

    /// Cancel current work
    fn cancel(&mut self) -> Result<(), ExternalSnarkWorkerError>;

    /// Kills external process.
    fn kill(&mut self) -> Result<(), ExternalSnarkWorkerError>;
}

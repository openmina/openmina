use std::ffi::OsStr;

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
    Error(ExternalSnarkWorkerError),
}

pub trait ExternalSnarkWorkerService {
    /// Starts external process.
    fn start<P: AsRef<OsStr>>(&mut self, path: P) -> Result<(), ExternalSnarkWorkerError>;

    /// Submits snark work
    fn submit(&mut self, spec: SnarkWorkSpec) -> Result<(), ExternalSnarkWorkerError>;

    /// Kills external process.
    fn kill(&mut self) -> Result<(), ExternalSnarkWorkerError>;
}

use serde::{Deserialize, Serialize};

use crate::block_producer::BlockProducerEvent;
use crate::external_snark_worker::ExternalSnarkWorkerEvent;
pub use crate::p2p::{P2pConnectionEvent, P2pEvent};
pub use crate::rpc::{RpcId, RpcRequest};
pub use crate::snark::SnarkEvent;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    P2p(P2pEvent),
    Snark(SnarkEvent),
    Rpc(RpcId, RpcRequest),
    ExternalSnarkWorker(ExternalSnarkWorkerEvent),
    BlockProducerEvent(BlockProducerEvent),
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P2p(v) => v.fmt(f),
            Self::Snark(v) => v.fmt(f),
            Self::Rpc(id, req) => {
                write!(f, "Rpc, {id}, ")?;
                match req {
                    RpcRequest::StateGet => write!(f, "StateGet"),
                    RpcRequest::ActionStatsGet(query) => write!(f, "ActionStatsGet, {query:?}"),
                    RpcRequest::SyncStatsGet(query) => write!(f, "SyncStatsGet, {query:?}"),
                    RpcRequest::P2pConnectionOutgoing(opts) => {
                        write!(f, "P2pConnectionOutgoing, {opts}")
                    }
                    RpcRequest::P2pConnectionIncoming(opts) => {
                        write!(f, "P2pConnectionIncoming, {}", opts.peer_id)
                    }
                    RpcRequest::ScanStateSummaryGet(query) => {
                        write!(f, "ScanStateSummaryGet, {query:?}")
                    }
                    RpcRequest::SnarkPoolGet => write!(f, "SnarkPoolGet"),
                    RpcRequest::SnarkPoolJobGet { job_id } => {
                        write!(f, "SnarkPoolJobGet, {job_id}")
                    }
                    RpcRequest::SnarkerConfig => write!(f, "SnarkerConfig"),
                    RpcRequest::SnarkerJobCommit { job_id } => {
                        write!(f, "SnarkerJobCommit, {job_id}")
                    }
                    RpcRequest::SnarkerJobSpec { job_id } => write!(f, "SnarkerJobSpec, {job_id}"),
                    RpcRequest::SnarkerWorkers => write!(f, "SnarkerWorkers"),
                    RpcRequest::HealthCheck => write!(f, "HealthCheck"),
                    RpcRequest::ReadinessCheck => write!(f, "ReadinessCheck"),
                }
            }
            Self::ExternalSnarkWorker(event) => {
                write!(f, "ExternalSnarkWorker, ")?;

                match event {
                    ExternalSnarkWorkerEvent::Started => write!(f, "Started"),
                    ExternalSnarkWorkerEvent::Killed => write!(f, "Killed"),
                    ExternalSnarkWorkerEvent::WorkResult(_) => write!(f, "WorkResult"),
                    ExternalSnarkWorkerEvent::WorkError(_) => write!(f, "WorkError"),
                    ExternalSnarkWorkerEvent::WorkCancelled => write!(f, "WorkCancelled"),
                    ExternalSnarkWorkerEvent::Error(_) => write!(f, "Error"),
                }
            },
            Self::BlockProducerEvent(event) => event.fmt(f),
        }
    }
}

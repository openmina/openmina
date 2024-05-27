use serde::{Deserialize, Serialize};

pub use crate::block_producer::BlockProducerEvent;
pub use crate::external_snark_worker::ExternalSnarkWorkerEvent;
pub use crate::ledger::LedgerEvent;
pub use crate::p2p::{P2pConnectionEvent, P2pEvent};
pub use crate::rpc::{RpcId, RpcRequest};
pub use crate::snark::SnarkEvent;

use crate::transition_frontier::genesis::GenesisConfigLoaded;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    P2p(P2pEvent),
    Ledger(LedgerEvent),
    Snark(SnarkEvent),
    Rpc(RpcId, Box<RpcRequest>),
    ExternalSnarkWorker(ExternalSnarkWorkerEvent),
    BlockProducerEvent(BlockProducerEvent),

    GenesisLoad(Result<GenesisConfigLoaded, String>),
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P2p(v) => v.fmt(f),
            Self::Ledger(v) => v.fmt(f),
            Self::Snark(v) => v.fmt(f),
            Self::Rpc(id, req) => {
                write!(f, "Rpc, {id}, ")?;
                match req.as_ref() {
                    RpcRequest::StateGet(filter) => write!(f, "StateGet, {filter:?}"),
                    RpcRequest::ActionStatsGet(query) => write!(f, "ActionStatsGet, {query:?}"),
                    RpcRequest::SyncStatsGet(query) => write!(f, "SyncStatsGet, {query:?}"),
                    RpcRequest::BlockProducerStatsGet => write!(f, "BlockProducerStatsGet"),
                    RpcRequest::PeersGet => write!(f, "PeersGet"),
                    RpcRequest::MessageProgressGet => write!(f, "MessageProgressGet"),
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
                    RpcRequest::DiscoveryRoutingTable => write!(f, "DiscoveryRoutingTable"),
                    RpcRequest::DiscoveryBoostrapStats => write!(f, "DiscoveryBoostrapStats"),
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
            }
            Self::BlockProducerEvent(event) => event.fmt(f),
            Self::GenesisLoad(res) => {
                write!(f, "GenesisLoad, ")?;
                match res {
                    Err(_) => {
                        write!(f, "Err")
                    }
                    Ok(data) => {
                        write!(f, "Ok, {}", data.ledger_hash)
                    }
                }
            }
        }
    }
}

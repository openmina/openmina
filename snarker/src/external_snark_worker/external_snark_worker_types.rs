use std::sync::Arc;

use mina_p2p_messages::v2::{
    SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse, TransactionSnarkWorkTStableV2Proofs,
};
use shared::snark_job_id::SnarkJobId;

pub type SnarkWorkId = SnarkJobId;

/// TODO use more slim type `OneOrTwo`...
pub type SnarkWorkSpec = SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse;

pub type SnarkWorkResult = Arc<TransactionSnarkWorkTStableV2Proofs>;

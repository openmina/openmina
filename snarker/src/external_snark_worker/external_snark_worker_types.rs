use std::sync::Arc;

use mina_p2p_messages::v2::{
    SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances,
    TransactionSnarkWorkTStableV2Proofs,
};
use openmina_core::snark::SnarkJobId;

pub type SnarkWorkId = SnarkJobId;

/// TODO use more slim type `OneOrTwo`...
pub type SnarkWorkSpec = SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances;

pub type SnarkWorkResult = Arc<TransactionSnarkWorkTStableV2Proofs>;

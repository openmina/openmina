use mina_p2p_messages::v2::{
    SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse,
    SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQuery,
};
use shared::snark_job_id::SnarkJobId;

/// TODO
pub type SnarkWorkId = SnarkJobId;

/// TODO
pub type SnarkWorkSpec = SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse;

/// TODO
pub type SnarkWorkResult = SnarkWorkerWorkerRpcsVersionedSubmitWorkV2TQuery;

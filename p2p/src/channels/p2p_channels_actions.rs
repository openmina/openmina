use serde::{Deserialize, Serialize};

use super::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;

pub type P2pChannelsActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pChannelsAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsAction {
    SnarkJobCommitment(P2pChannelsSnarkJobCommitmentAction),
}

impl P2pChannelsAction {
    pub fn peer_id(&self) -> &crate::PeerId {
        match self {
            Self::SnarkJobCommitment(v) => v.peer_id(),
        }
    }
}

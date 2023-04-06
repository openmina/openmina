use super::*;

impl redux::EnablingCondition<crate::State>
    for snark_job_commitment::P2pChannelsSnarkJobCommitmentInitAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State>
    for snark_job_commitment::P2pChannelsSnarkJobCommitmentPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State>
    for snark_job_commitment::P2pChannelsSnarkJobCommitmentReadyAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

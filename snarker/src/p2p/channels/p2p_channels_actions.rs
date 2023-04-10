use super::*;

impl redux::EnablingCondition<crate::State> for P2pChannelsMessageReceivedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

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

impl redux::EnablingCondition<crate::State>
    for snark_job_commitment::P2pChannelsSnarkJobCommitmentRequestSendAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State>
    for snark_job_commitment::P2pChannelsSnarkJobCommitmentPromiseReceivedAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State>
    for snark_job_commitment::P2pChannelsSnarkJobCommitmentReceivedAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State>
    for snark_job_commitment::P2pChannelsSnarkJobCommitmentRequestReceivedAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State>
    for snark_job_commitment::P2pChannelsSnarkJobCommitmentResponseSendAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

use super::{P2pChannelsAction, P2pChannelsActionWithMetaRef, P2pChannelsState};

impl P2pChannelsState {
    pub fn reducer(&mut self, action: P2pChannelsActionWithMetaRef<'_>, is_libp2p: bool) {
        let (action, meta) = action.split();
        match action {
            P2pChannelsAction::MessageReceived(_) => {}
            P2pChannelsAction::BestTip(action) => {
                self.best_tip.reducer(meta.with_action(action), is_libp2p);
            }
            P2pChannelsAction::Transaction(action) => {
                self.transaction.reducer(meta.with_action(action));
            }
            P2pChannelsAction::Snark(action) => {
                self.snark.reducer(meta.with_action(action));
            }
            P2pChannelsAction::SnarkJobCommitment(action) => {
                self.snark_job_commitment.reducer(meta.with_action(action));
            }
            P2pChannelsAction::Rpc(action) => {
                self.rpc
                    .reducer(meta.with_action(action), &mut self.next_local_rpc_id);
            }
            P2pChannelsAction::StreamingRpc(action) => {
                self.streaming_rpc
                    .reducer(meta.with_action(action), &mut self.next_local_rpc_id);
            }
        }
    }
}

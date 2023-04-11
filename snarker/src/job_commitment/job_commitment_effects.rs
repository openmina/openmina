use crate::p2p::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentResponseSendAction;
use crate::{Service, Store};

use super::{JobCommitmentAction, JobCommitmentActionWithMeta, JobCommitmentP2pSendAction};

pub fn job_commitment_effects<S: Service>(
    store: &mut Store<S>,
    action: JobCommitmentActionWithMeta,
) {
    let (action, _) = action.split();

    match action {
        JobCommitmentAction::Add(_) => {}
        JobCommitmentAction::P2pSendAll(_) => {
            for peer_id in store.state().p2p.ready_peers() {
                store.dispatch(JobCommitmentP2pSendAction { peer_id });
            }
        }
        JobCommitmentAction::P2pSend(a) => {
            let state = store.state();
            let Some(peer) = state.p2p.get_ready_peer(&a.peer_id) else { return };
            let (index, limit) = peer
                .channels
                .snark_job_commitment
                .next_send_index_and_limit();

            let iter = state.job_commitments.range(index..).take(limit as usize);

            let mut commitments = vec![];
            let mut first_index = None;
            let mut last_index = None;

            for (index, commitment) in iter {
                first_index = first_index.or(Some(index));
                last_index = Some(index);
                commitments.push(commitment.commitment.clone());
            }
            let Some(first_index) = first_index else { return };
            let Some(last_index) = last_index else { return };

            store.dispatch(P2pChannelsSnarkJobCommitmentResponseSendAction {
                peer_id: a.peer_id,
                commitments,
                first_index,
                last_index,
            });
        }
    }
}

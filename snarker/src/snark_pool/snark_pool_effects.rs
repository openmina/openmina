use crate::external_snark_worker::ExternalSnarkWorkerSubmitWorkAction;
use crate::p2p::channels::snark::{
    P2pChannelsSnarkLibp2pBroadcastAction, P2pChannelsSnarkResponseSendAction,
};
use crate::p2p::channels::snark_job_commitment::{
    P2pChannelsSnarkJobCommitmentResponseSendAction, SnarkJobCommitment,
};
use crate::{Service, State, Store};

use super::{
    JobState, SnarkPoolAction, SnarkPoolActionWithMeta, SnarkPoolAutoCreateCommitmentAction,
    SnarkPoolCommitmentCreateAction, SnarkPoolJobCommitmentAddAction,
    SnarkPoolJobCommitmentTimeoutAction, SnarkPoolP2pSendAction,
};

pub fn job_commitment_effects<S: Service>(store: &mut Store<S>, action: SnarkPoolActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        SnarkPoolAction::JobsUpdate(_) => {
            store.dispatch(SnarkPoolAutoCreateCommitmentAction {});
        }
        SnarkPoolAction::AutoCreateCommitment(_) => {
            let state = store.state();
            let available_workers = state.external_snark_worker.available();
            let available_jobs = state.snark_pool.available_jobs_iter();
            let job_ids = available_jobs
                .take(available_workers)
                .map(|job| job.id.clone())
                .collect::<Vec<_>>();
            for job_id in job_ids {
                store.dispatch(SnarkPoolCommitmentCreateAction { job_id });
            }
        }
        SnarkPoolAction::CommitmentCreate(a) => {
            let timestamp_ms = meta.time_as_nanos() / 1_000_000;
            let config = &store.state().config;
            store.dispatch(SnarkPoolJobCommitmentAddAction {
                commitment: SnarkJobCommitment::new(
                    timestamp_ms,
                    a.job_id.clone(),
                    config.fee.clone(),
                    config.public_key.clone().into(),
                ),
                sender: store.state().p2p.config.identity_pub_key.peer_id(),
            });
            store.dispatch(ExternalSnarkWorkerSubmitWorkAction { job_id: a.job_id });
        }
        SnarkPoolAction::CommitmentAdd(_) => {}
        SnarkPoolAction::WorkAdd(a) => {
            // TODO(binier): only broadcast after validation
            store.dispatch(P2pChannelsSnarkLibp2pBroadcastAction { snark: a.snark });
        }
        SnarkPoolAction::P2pSendAll(_) => {
            for peer_id in store.state().p2p.ready_peers() {
                store.dispatch(SnarkPoolP2pSendAction { peer_id });
            }
        }
        SnarkPoolAction::P2pSend(a) => {
            let state = store.state();
            let Some(peer) = state.p2p.get_ready_peer(&a.peer_id) else {
                return;
            };

            // Send commitments.
            let index_and_limit = peer
                .channels
                .snark_job_commitment
                .next_send_index_and_limit();
            let (commitments, first_index, last_index) =
                data_to_send(state, index_and_limit, |job| job.commitment_msg().cloned());

            let send_commitments = P2pChannelsSnarkJobCommitmentResponseSendAction {
                peer_id: a.peer_id,
                commitments,
                first_index,
                last_index,
            };

            // Send snarks.
            let index_and_limit = peer.channels.snark.next_send_index_and_limit();
            let (snarks, first_index, last_index) =
                data_to_send(state, index_and_limit, |job| job.snark_msg());

            store.dispatch(send_commitments);
            store.dispatch(P2pChannelsSnarkResponseSendAction {
                peer_id: a.peer_id,
                snarks,
                first_index,
                last_index,
            });
        }
        SnarkPoolAction::CheckTimeouts(_) => {
            let timed_out_ids = store
                .state()
                .snark_pool
                .timed_out_commitments_iter(meta.time())
                .cloned()
                .collect::<Vec<_>>();
            for job_id in timed_out_ids {
                store.dispatch(SnarkPoolJobCommitmentTimeoutAction { job_id });
            }
        }
        SnarkPoolAction::JobCommitmentTimeout(_) => {}
    }
}

pub fn data_to_send<F, T>(
    state: &State,
    (index, limit): (u64, u8),
    get_data: F,
) -> (Vec<T>, u64, u64)
where
    F: Fn(&JobState) -> Option<T>,
{
    if limit == 0 {
        let index = index.saturating_sub(1);
        return (vec![], index, index);
    }

    state
        .snark_pool
        .range(index..)
        .try_fold(
            (vec![], None),
            |(mut list, mut first_index), (index, job)| {
                if let Some(data) = get_data(job) {
                    let first_index = *first_index.get_or_insert(index);
                    list.push(data);
                    if list.len() >= limit as usize {
                        return Err((list, first_index, index));
                    }
                }

                Ok((list, first_index))
            },
        )
        // Loop iterated on whole snark pool.
        .map(|(list, first_index)| {
            let snark_pool_last_index = state.snark_pool.last_index();
            (list, first_index.unwrap_or(index), snark_pool_last_index)
        })
        // Loop preemptively ended.
        .unwrap_or_else(|v| v)
}

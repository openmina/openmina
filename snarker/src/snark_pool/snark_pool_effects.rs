use binprot::BinProtWrite;
use ledger::scan_state::conv::job_to_spec;
use mina_p2p_messages::v2::{StateBodyHash, UnsignedExtendedUInt64Int64ForVersionTagsStableV1};

use crate::p2p::channels::snark_job_commitment::{
    P2pChannelsSnarkJobCommitmentResponseSendAction, SnarkJobCommitment,
};
use crate::{Service, Store};

use super::{
    SnarkPoolAction, SnarkPoolActionWithMeta, SnarkPoolJobCommitmentAddAction,
    SnarkPoolJobCommitmentTimeoutAction, SnarkPoolP2pSendAction,
};

pub fn job_commitment_effects<S: Service>(store: &mut Store<S>, action: SnarkPoolActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        SnarkPoolAction::CommitmentCreate(a) => {
            let timestamp_ms = meta.time_as_nanos() / 1_000_000;
            let pub_key = store.state().config.public_key.clone();
            store.dispatch(SnarkPoolJobCommitmentAddAction {
                commitment: SnarkJobCommitment::new(
                    timestamp_ms,
                    a.job_id.clone(),
                    pub_key.clone().into(),
                ),
                sender: store.state().p2p.config.identity_pub_key.peer_id(),
            });

            let path = std::path::PathBuf::from(&format!("work-{}", a.job_id));

            let Some(job) = store.state().snark_pool.get(&a.job_id) else {
                panic!();
            };
            let protocol_state_body = |block_hash: StateBodyHash| {
                store
                    .state()
                    .transition_frontier
                    .best_chain
                    .iter()
                    .find_map(|block_with_hash| {
                        if block_with_hash.block.header.protocol_state.body.hash() == *block_hash {
                            Some(block_with_hash.block.header.protocol_state.body.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            };
            let input = job_to_spec(
                pub_key.into(),
                mina_p2p_messages::v2::CurrencyFeeStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(1_000_000_000_u64.into()),
                ),
                job.job.clone(),
                &protocol_state_body,
            );
            std::fs::File::create(&path.with_extension(".bin"))
                .and_then(|mut w| input.binprot_write(&mut w))
                .unwrap();
            std::fs::File::create(&path.with_extension(".json"))
                .and_then(|w| {
                    serde_json::to_writer(w, &input).unwrap();
                    Ok(())
                })
                .unwrap();

            // TODO(akoptelov): start working on this job.
        }
        SnarkPoolAction::CommitmentAdd(_) => {}
        SnarkPoolAction::JobsUpdate(_) => {}
        SnarkPoolAction::P2pSendAll(_) => {
            for peer_id in store.state().p2p.ready_peers() {
                store.dispatch(SnarkPoolP2pSendAction { peer_id });
            }
        }
        SnarkPoolAction::P2pSend(a) => {
            let state = store.state();
            let Some(peer) = state.p2p.get_ready_peer(&a.peer_id) else { return };
            let (index, limit) = peer
                .channels
                .snark_job_commitment
                .next_send_index_and_limit();

            let iter = state
                .snark_pool
                .range(index..)
                .filter_map(|(index, job)| Some((index, job.commitment.as_ref()?)))
                .take(limit as usize);

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

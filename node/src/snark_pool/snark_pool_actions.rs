use ledger::scan_state::scan_state::transaction_snark::OneOrTwo;
use ledger::scan_state::scan_state::AvailableJobMessage;
use openmina_core::snark::{Snark, SnarkJobCommitment, SnarkJobId};
use serde::{Deserialize, Serialize};

use crate::p2p::PeerId;

use super::candidate::SnarkPoolCandidateAction;
use super::SnarkWork;

pub type SnarkPoolActionWithMeta = redux::ActionWithMeta<SnarkPoolAction>;
pub type SnarkPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkPoolAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkPoolAction {
    Candidate(SnarkPoolCandidateAction),

    JobsUpdate {
        jobs: Vec<OneOrTwo<AvailableJobMessage>>,
        orphaned_snarks: Vec<SnarkWork>,
    },
    AutoCreateCommitment,
    CommitmentCreate {
        job_id: SnarkJobId,
    },
    CommitmentAdd {
        commitment: SnarkJobCommitment,
        sender: PeerId,
    },
    WorkAdd {
        snark: Snark,
        sender: PeerId,
    },
    P2pSendAll,
    P2pSend {
        peer_id: PeerId,
    },
    CheckTimeouts,
    JobCommitmentTimeout {
        job_id: SnarkJobId,
    },
}

impl redux::EnablingCondition<crate::State> for SnarkPoolAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            SnarkPoolAction::Candidate(action) => action.is_enabled(state, time),
            SnarkPoolAction::AutoCreateCommitment => state
                .config
                .snarker
                .as_ref()
                .map_or(false, |v| v.auto_commit),
            SnarkPoolAction::CommitmentCreate { job_id } => {
                state.config.snarker.is_some() && state.snark_pool.should_create_commitment(job_id)
            }
            SnarkPoolAction::CommitmentAdd { commitment, .. } => state
                .snark_pool
                .get(&commitment.job_id)
                .map_or(false, |s| match s.commitment.as_ref() {
                    Some(cur) => commitment > &cur.commitment,
                    None => true,
                }),
            SnarkPoolAction::WorkAdd { snark, .. } => {
                state
                    .snark_pool
                    .get(&snark.job_id())
                    .map_or(false, |s| match s.snark.as_ref() {
                        Some(cur) => snark > &cur.work,
                        None => true,
                    })
            }
            SnarkPoolAction::P2pSend { peer_id } => state
                .p2p
                .get_ready_peer(peer_id)
                // Only send commitments/snarks if peer has the same best tip,
                // or its best tip is extension of our best tip. In such case
                // no commitment/snark will be dropped by peer, because it
                // doesn't yet have those jobs.
                //
                // By sending commitments/snarks to the peer, which has next
                // best tip, we might send outdated commitments/snarks, but
                // we might send useful ones as well.
                .and_then(|p| {
                    let peer_best_tip = p.best_tip.as_ref()?;
                    let our_best_tip = state.transition_frontier.best_tip()?.hash();
                    Some(p).filter(|_| {
                        peer_best_tip.hash() == our_best_tip
                            || peer_best_tip.pred_hash() == our_best_tip
                    })
                })
                .map_or(false, |p| {
                    let check =
                        |(next_index, limit), last_index| limit > 0 && next_index <= last_index;
                    let last_index = state.snark_pool.last_index();

                    check(
                        p.channels.snark_job_commitment.next_send_index_and_limit(),
                        last_index,
                    ) || check(p.channels.snark.next_send_index_and_limit(), last_index)
                }),
            SnarkPoolAction::CheckTimeouts => time
                .checked_sub(state.snark_pool.last_check_timeouts)
                .map_or(false, |dur| dur.as_secs() >= 5),
            SnarkPoolAction::JobCommitmentTimeout { job_id } => {
                state.snark_pool.is_commitment_timed_out(job_id, time)
            }
            SnarkPoolAction::JobsUpdate { .. } => true,
            SnarkPoolAction::P2pSendAll => true,
        }
    }
}

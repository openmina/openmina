use std::cmp::Ordering;

use ledger::scan_state::scan_state::transaction_snark::OneOrTwo;
use ledger::scan_state::scan_state::AvailableJobMessage;
use serde::{Deserialize, Serialize};
use shared::snark::Snark;
use shared::snark_job_id::SnarkJobId;

use crate::p2p::channels::snark_job_commitment::SnarkJobCommitment;
use crate::p2p::PeerId;

pub type SnarkPoolActionWithMeta = redux::ActionWithMeta<SnarkPoolAction>;
pub type SnarkPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkPoolAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkPoolAction {
    JobsUpdate(SnarkPoolJobsUpdateAction),
    AutoCreateCommitment(SnarkPoolAutoCreateCommitmentAction),
    CommitmentCreate(SnarkPoolCommitmentCreateAction),
    CommitmentAdd(SnarkPoolJobCommitmentAddAction),
    WorkAdd(SnarkPoolWorkAddAction),
    P2pSendAll(SnarkPoolP2pSendAllAction),
    P2pSend(SnarkPoolP2pSendAction),
    CheckTimeouts(SnarkPoolCheckTimeoutsAction),
    JobCommitmentTimeout(SnarkPoolJobCommitmentTimeoutAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolJobsUpdateAction {
    pub jobs: Vec<OneOrTwo<AvailableJobMessage>>,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolJobsUpdateAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolAutoCreateCommitmentAction {}

impl redux::EnablingCondition<crate::State> for SnarkPoolAutoCreateCommitmentAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &crate::State) -> bool {
        state.config.job_commitments.auto_commit
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCommitmentCreateAction {
    pub job_id: SnarkJobId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCommitmentCreateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.snark_pool.should_create_commitment(&self.job_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolJobCommitmentAddAction {
    pub commitment: SnarkJobCommitment,
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolJobCommitmentAddAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .snark_pool
            .get(&self.commitment.job_id)
            .map_or(false, |s| match s.commitment.as_ref() {
                Some(cur) => {
                    let cur_fee = cur.commitment.fee.0.as_u64();
                    let new_fee = self.commitment.fee.0.as_u64();
                    match cur_fee.cmp(&new_fee) {
                        Ordering::Less => false,
                        Ordering::Greater => true,
                        Ordering::Equal => {
                            self.commitment.tie_breaker_hash() > cur.commitment.tie_breaker_hash()
                        }
                    }
                }
                None => true,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolWorkAddAction {
    pub snark: Snark,
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolWorkAddAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .snark_pool
            .get(&self.snark.job_id())
            .map_or(false, |s| match s.snark.as_ref() {
                Some(cur) => cur.work.fee.0.as_u64() > self.snark.fee.0.as_u64(),
                None => true,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolP2pSendAllAction {}

impl redux::EnablingCondition<crate::State> for SnarkPoolP2pSendAllAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolP2pSendAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolP2pSendAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .p2p
            .get_ready_peer(&self.peer_id)
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
                let check = |(next_index, limit), last_index| limit > 0 && next_index <= last_index;
                let last_index = state.snark_pool.last_index();

                check(
                    p.channels.snark_job_commitment.next_send_index_and_limit(),
                    last_index,
                ) || check(p.channels.snark.next_send_index_and_limit(), last_index)
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCheckTimeoutsAction {}

impl redux::EnablingCondition<crate::State> for SnarkPoolCheckTimeoutsAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .time()
            .checked_sub(state.snark_pool.last_check_timeouts)
            .map_or(false, |dur| dur.as_secs() >= 5)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolJobCommitmentTimeoutAction {
    pub job_id: SnarkJobId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolJobCommitmentTimeoutAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .snark_pool
            .is_commitment_timed_out(&self.job_id, state.time())
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::SnarkPool(value.into())
            }
        }
    };
}

impl_into_global_action!(SnarkPoolJobsUpdateAction);
impl_into_global_action!(SnarkPoolAutoCreateCommitmentAction);
impl_into_global_action!(SnarkPoolCommitmentCreateAction);
impl_into_global_action!(SnarkPoolJobCommitmentAddAction);
impl_into_global_action!(SnarkPoolWorkAddAction);
impl_into_global_action!(SnarkPoolP2pSendAllAction);
impl_into_global_action!(SnarkPoolP2pSendAction);
impl_into_global_action!(SnarkPoolCheckTimeoutsAction);
impl_into_global_action!(SnarkPoolJobCommitmentTimeoutAction);

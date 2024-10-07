use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::snark_job_commitment_effectful::P2pChannelsSnarkJobCommitmentEffectfulAction,
    P2pState,
};

use super::{
    P2pChannelsSnarkJobCommitmentAction, P2pChannelsSnarkJobCommitmentState,
    SnarkJobCommitmentPropagationState,
};

const LIMIT: u8 = 16;

impl P2pChannelsSnarkJobCommitmentState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsSnarkJobCommitmentAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;
        let peer_id = *action.peer_id();
        let snark_job_state = &mut p2p_state
            .get_ready_peer_mut(&peer_id)
            .ok_or_else(|| format!("Peer state not found for: {action:?}"))?
            .channels
            .snark_job_commitment;

        match action {
            P2pChannelsSnarkJobCommitmentAction::Init { .. } => {
                *snark_job_state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSnarkJobCommitmentAction::Pending { peer_id });
                Ok(())
            }
            P2pChannelsSnarkJobCommitmentAction::Pending { .. } => {
                *snark_job_state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsSnarkJobCommitmentAction::Ready { .. } => {
                *snark_job_state = Self::Ready {
                    time: meta.time(),
                    local: SnarkJobCommitmentPropagationState::WaitingForRequest {
                        time: meta.time(),
                    },
                    remote: SnarkJobCommitmentPropagationState::WaitingForRequest {
                        time: meta.time(),
                    },
                    next_send_index: 0,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSnarkJobCommitmentAction::RequestSend {
                    peer_id,
                    limit: LIMIT,
                });
                Ok(())
            }
            P2pChannelsSnarkJobCommitmentAction::RequestSend { limit, .. } => {
                let Self::Ready { local, .. } = snark_job_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkJobCommitmentAction::RequestSend`, state: {:?}",
                        snark_job_state
                    );
                    return Ok(());
                };
                *local = SnarkJobCommitmentPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSnarkJobCommitmentAction::RequestSend {
                    peer_id,
                    limit: *limit,
                });
                Ok(())
            }
            P2pChannelsSnarkJobCommitmentAction::PromiseReceived { promised_count, .. } => {
                let Self::Ready { local, .. } = snark_job_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkJobCommitmentAction::PromiseReceived`, state: {:?}",
                        snark_job_state
                    );
                    return Ok(());
                };
                let SnarkJobCommitmentPropagationState::Requested {
                    requested_limit, ..
                } = &local
                else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkJobCommitmentAction::PromiseReceived`, state: {:?}",
                        snark_job_state
                    );
                    return Ok(());
                };

                *local = SnarkJobCommitmentPropagationState::Responding {
                    time: meta.time(),
                    requested_limit: *requested_limit,
                    promised_count: *promised_count,
                    current_count: 0,
                };
                Ok(())
            }
            P2pChannelsSnarkJobCommitmentAction::Received { commitment, .. } => {
                let Self::Ready { local, .. } = snark_job_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkJobCommitmentAction::Received`, state: {:?}",
                        snark_job_state
                    );
                    return Ok(());
                };
                let SnarkJobCommitmentPropagationState::Responding {
                    promised_count,
                    current_count,
                    ..
                } = local
                else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkJobCommitmentAction::Received`, state: {:?}",
                        snark_job_state
                    );
                    return Ok(());
                };

                *current_count += 1;

                if current_count >= promised_count {
                    *local = SnarkJobCommitmentPropagationState::Responded {
                        time: meta.time(),
                        count: *current_count,
                    };
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                dispatcher.push(P2pChannelsSnarkJobCommitmentAction::RequestSend {
                    peer_id,
                    limit: LIMIT,
                });

                if let Some(callback) = &p2p_state
                    .callbacks
                    .on_p2p_channels_snark_job_commitment_received
                {
                    dispatcher.push_callback(callback.clone(), (peer_id, commitment.clone()));
                }
                Ok(())
            }
            P2pChannelsSnarkJobCommitmentAction::RequestReceived { limit, .. } => {
                let Self::Ready { remote, .. } = snark_job_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkJobCommitmentAction::RequestReceived`, state: {:?}",
                        snark_job_state
                    );
                    return Ok(());
                };
                *remote = SnarkJobCommitmentPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
                Ok(())
            }
            P2pChannelsSnarkJobCommitmentAction::ResponseSend {
                last_index,
                commitments,
                ..
            } => {
                let Self::Ready {
                    remote,
                    next_send_index,
                    ..
                } = snark_job_state
                else {
                    bug_condition!(
                    "Invalid state for `P2pChannelsSnarkJobCommitmentAction::ResponseSend`, state: {:?}",
                    snark_job_state
                );
                    return Ok(());
                };
                *next_send_index = last_index + 1;

                let count = commitments.len() as u8;
                if count == 0 {
                    return Ok(());
                }

                *remote = SnarkJobCommitmentPropagationState::Responded {
                    time: meta.time(),
                    count,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSnarkJobCommitmentEffectfulAction::ResponseSend {
                    peer_id,
                    commitments: commitments.clone(),
                });
                Ok(())
            }
        }
    }
}

use ledger::scan_state::scan_state::{
    transaction_snark::{OneOrTwo, Statement},
    AvailableJobMessage,
};
use mina_p2p_messages::v2::{
    SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances,
    SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single, StateBodyHash,
    TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    TransactionSnarkScanStateTransactionWithWitnessStableV2,
};
use serde::{Deserialize, Serialize};

use crate::transition_frontier::TransitionFrontierState;

#[derive(Clone, Debug, derive_more::From, Serialize, Deserialize, thiserror::Error)]
pub enum SnarkWorkSpecError {
    #[error("unknown state body hash: {_0}")]
    UnknownStateBodyHash(StateBodyHash),
    #[error("error merging statements: {_0}")]
    MergeStatementError(String),
    #[error("Invalid BigInt")]
    InvalidBigInt,
}

pub fn available_job_to_snark_worker_spec(
    job: OneOrTwo<AvailableJobMessage>,
    transition_frontier: &TransitionFrontierState,
) -> Result<SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances, SnarkWorkSpecError> {
    Ok(match job {
        OneOrTwo::One(v) => SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances::One(
            with_merged_statement(v, transition_frontier)?,
        ),
        OneOrTwo::Two((v1, v2)) => {
            SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Instances::Two((
                with_merged_statement(v1, transition_frontier)?,
                with_merged_statement(v2, transition_frontier)?,
            ))
        }
    })
}

/// Converts [AvailableJobMessage] instance to the specification suitable for Mina snark worker.
fn with_merged_statement(
    job: AvailableJobMessage,
    transition_frontier: &TransitionFrontierState,
) -> Result<SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single, SnarkWorkSpecError> {
    match job {
        AvailableJobMessage::Base(TransactionSnarkScanStateTransactionWithWitnessStableV2 {
            transaction_with_info,
            state_hash,
            statement,
            init_stack,
            first_pass_ledger_witness,
            second_pass_ledger_witness,
            block_global_slot,
        }) => {
            let (transaction, status) = transaction_with_info.varying.into();
            let mina_p2p_messages::v2::MinaStateSnarkedLedgerStatePendingCoinbaseStackStateInitStackStableV1::Base(init_stack) = init_stack else {
                panic!("merge in base transaction");
            };
            let protocol_state_body = transition_frontier
                .get_state_body(&state_hash.0)
                .ok_or_else(|| SnarkWorkSpecError::UnknownStateBodyHash(state_hash.1.clone()))?
                .clone();
            let transaction_witness = mina_p2p_messages::v2::TransactionWitnessStableV2 {
                transaction,
                first_pass_ledger: first_pass_ledger_witness,
                second_pass_ledger: second_pass_ledger_witness,
                protocol_state_body,
                init_stack,
                status,
                block_global_slot,
            };
            Ok(
                SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Transition(
                    statement,
                    transaction_witness,
                ),
            )
        }
        AvailableJobMessage::Merge {
            left:
                TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(ledger_proof1, _message1),
            right:
                TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(ledger_proof2, _message2),
        } => {
            let (Ok(ledger_stmt1), Ok(ledger_stmt2)) = (
                Statement::<()>::try_from(&ledger_proof1.statement),
                Statement::<()>::try_from(&ledger_proof2.statement),
            ) else {
                return Err(SnarkWorkSpecError::InvalidBigInt);
            };
            let merged_stmt = ledger_stmt1
                .merge(&ledger_stmt2)
                .map_err(SnarkWorkSpecError::MergeStatementError)?;
            Ok(
                SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Merge(Box::new((
                    (&merged_stmt).into(),
                    ledger_proof1,
                    ledger_proof2,
                ))),
            )
        }
    }
}

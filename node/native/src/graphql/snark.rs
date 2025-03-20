use juniper::GraphQLObject;
use ledger::scan_state::scan_state::{AvailableJobMessage, ParallelScanAvailableJob};
use mina_p2p_messages::v2::{
    MinaBaseFeeExcessStableV1, MinaStateBlockchainStateValueStableV2SignedAmount,
    TransactionSnarkScanStateTransactionWithWitnessStableV2, TransactionSnarkStableV2,
};
use node::snark_pool::JobState;

use super::ConversionError;

#[derive(GraphQLObject, Debug)]
#[graphql(description = "A Mina block")]
pub struct GraphQLPendingSnarkWork {
    /// Work bundle with one or two snark work
    pub work_bundle: Vec<GraphQLWorkDescription>,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLWorkDescription {
    /// Base58Check-encoded hash of the source first-pass ledger
    pub source_first_pass_ledger_hash: String,
    /// Base58Check-encoded hash of the target first-pass ledger
    pub target_first_pass_ledger_hash: String,
    /// Base58Check-encoded hash of the source second-pass ledger
    pub source_second_pass_ledger_hash: String,
    /// Base58Check-encoded hash of the target second-pass ledger
    pub target_second_pass_ledger_hash: String,
    /// Total transaction fee that is not accounted for in the transition from source ledger to target ledger
    pub fee_excess: GraphQLFeeExcesses,
    /// Increase/Decrease in total supply
    pub supply_change: GraphQLSupplyChange,
    /// Increase in total supply
    pub supply_increase: String,
    /// Unique identifier for a snark work
    pub work_id: i32,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLFeeExcesses {
    pub fee_token_left: String,
    pub fee_excess_left: GraphQLFeeExcess,
    pub fee_token_right: String,
    pub fee_excess_right: GraphQLFeeExcess,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLFeeExcess {
    pub fee_magnitude: String,
    pub sign: String,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLSupplyChange {
    pub amount_magnitude: String,
    pub sign: String,
}

impl TryFrom<JobState> for GraphQLPendingSnarkWork {
    type Error = ConversionError;

    fn try_from(value: JobState) -> Result<Self, Self::Error> {
        let mut work_bundle = Vec::new();

        for job in value.job.into_iter() {
            work_bundle.push(GraphQLWorkDescription::try_from(job)?);
        }

        Ok(Self { work_bundle })
    }
}

impl TryFrom<AvailableJobMessage> for GraphQLWorkDescription {
    type Error = ConversionError;

    fn try_from(value: AvailableJobMessage) -> Result<Self, Self::Error> {
        match value {
            ParallelScanAvailableJob::Base(base) => GraphQLWorkDescription::try_from(base),
            ParallelScanAvailableJob::Merge { left, .. } => {
                GraphQLWorkDescription::try_from(left.0 .0)
            }
        }
    }
}

impl TryFrom<MinaBaseFeeExcessStableV1> for GraphQLFeeExcesses {
    type Error = ConversionError;

    fn try_from(value: MinaBaseFeeExcessStableV1) -> Result<Self, Self::Error> {
        Ok(Self {
            fee_token_left: value.0.token.to_string(),
            fee_excess_left: GraphQLFeeExcess {
                fee_magnitude: value.0.amount.magnitude.to_string(),
                sign: value.0.amount.sgn.to_string(),
            },
            fee_token_right: value.1.token.to_string(),
            fee_excess_right: GraphQLFeeExcess {
                fee_magnitude: value.1.amount.magnitude.to_string(),
                sign: value.1.amount.sgn.to_string(),
            },
        })
    }
}

impl TryFrom<MinaStateBlockchainStateValueStableV2SignedAmount> for GraphQLSupplyChange {
    type Error = ConversionError;

    fn try_from(
        value: MinaStateBlockchainStateValueStableV2SignedAmount,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            amount_magnitude: value.magnitude.to_string(),
            sign: value.sgn.to_string(),
        })
    }
}

impl TryFrom<TransactionSnarkScanStateTransactionWithWitnessStableV2> for GraphQLWorkDescription {
    type Error = ConversionError;

    fn try_from(
        value: TransactionSnarkScanStateTransactionWithWitnessStableV2,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            source_first_pass_ledger_hash: value.statement.source.first_pass_ledger.to_string(),
            target_first_pass_ledger_hash: value.statement.target.first_pass_ledger.to_string(),
            source_second_pass_ledger_hash: value.statement.source.second_pass_ledger.to_string(),
            target_second_pass_ledger_hash: value.statement.target.second_pass_ledger.to_string(),
            fee_excess: GraphQLFeeExcesses::try_from(value.statement.fee_excess.clone())?,
            supply_change: GraphQLSupplyChange::try_from(value.statement.supply_increase.clone())?,
            supply_increase: value.statement.supply_increase.magnitude.to_string(),
            work_id: 0,
        })
    }
}

impl TryFrom<TransactionSnarkStableV2> for GraphQLWorkDescription {
    type Error = ConversionError;

    fn try_from(value: TransactionSnarkStableV2) -> Result<Self, Self::Error> {
        Ok(Self {
            source_first_pass_ledger_hash: value.statement.source.first_pass_ledger.to_string(),
            target_first_pass_ledger_hash: value.statement.target.first_pass_ledger.to_string(),
            source_second_pass_ledger_hash: value.statement.source.second_pass_ledger.to_string(),
            target_second_pass_ledger_hash: value.statement.target.second_pass_ledger.to_string(),
            fee_excess: GraphQLFeeExcesses::try_from(value.statement.fee_excess.clone())?,
            supply_change: GraphQLSupplyChange::try_from(value.statement.supply_increase.clone())?,
            supply_increase: value.statement.supply_increase.magnitude.to_string(),
            work_id: 0,
        })
    }
}

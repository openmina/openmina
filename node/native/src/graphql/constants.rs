use juniper::GraphQLObject;
use openmina_core::{consensus::ConsensusConstants, constants::ConstraintConstants};

use super::ConversionError;

#[derive(GraphQLObject, Debug)]
pub struct GraphQLGenesisConstants {
    pub genesis_timestamp: String,
    pub coinbase: String,
    pub account_creation_fee: String,
}

impl GraphQLGenesisConstants {
    pub fn try_new(
        constrain_constants: ConstraintConstants,
        consensus_constants: ConsensusConstants,
    ) -> Result<Self, ConversionError> {
        Ok(GraphQLGenesisConstants {
            genesis_timestamp: consensus_constants
                .human_readable_genesis_timestamp()
                .map_err(|e| ConversionError::Custom(e.to_string()))?,
            coinbase: constrain_constants.coinbase_amount.to_string(),
            account_creation_fee: constrain_constants.account_creation_fee.to_string(),
        })
    }
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLDaemonStatus {
    pub consensus_configuration: GraphQLConsensusConfiguration,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLConsensusConfiguration {
    pub epoch_duration: i32,
    pub k: i32,
    pub slot_duration: i32,
    pub slots_per_epoch: i32,
}

impl From<ConsensusConstants> for GraphQLConsensusConfiguration {
    fn from(consensus_constants: ConsensusConstants) -> Self {
        GraphQLConsensusConfiguration {
            epoch_duration: consensus_constants.epoch_duration as i32,
            k: consensus_constants.k as i32,
            slot_duration: consensus_constants.slot_duration_ms as i32,
            slots_per_epoch: consensus_constants.slots_per_epoch as i32,
        }
    }
}

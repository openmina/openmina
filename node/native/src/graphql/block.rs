use juniper::GraphQLObject;
use openmina_core::block::AppliedBlock;

use crate::graphql::zkapp::{GraphQLFailureReason, GraphQLFeePayer, GraphQLZkappCommand};

use super::{zkapp::GraphQLZkapp, ConversionError};

#[derive(GraphQLObject, Debug)]
#[graphql(description = "A Mina block")]
pub struct GraphQLBestChainBlock {
    pub protocol_state: GraphQLProtocolState,
    pub state_hash: String,
    pub transactions: GraphQLTransactions,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLTransactions {
    pub zkapp_commands: Vec<GraphQLZkapp>,
}

impl TryFrom<AppliedBlock> for GraphQLBestChainBlock {
    type Error = ConversionError;
    fn try_from(value: AppliedBlock) -> Result<Self, Self::Error> {
        let block = value.block;
        let blockchain_state = GraphQLBlockchainState {
            snarked_ledger_hash: block.snarked_ledger_hash().to_string(),
            staged_ledger_hash: block
                .staged_ledger_hashes()
                .non_snark
                .ledger_hash
                .to_string(),
            date: block
                .header()
                .protocol_state
                .body
                .blockchain_state
                .timestamp
                .to_string(),
            utc_date: block
                .header()
                .protocol_state
                .body
                .blockchain_state
                .timestamp
                .to_string(),
            staged_ledger_proof_emitted: value.just_emitted_a_proof,
        };

        let protocol_state = GraphQLProtocolState {
            previous_state_hash: block.pred_hash().to_string(),
            blockchain_state,
            consensus_state: block
                .header()
                .protocol_state
                .body
                .consensus_state
                .clone()
                .into(),
        };

        Ok(Self {
            protocol_state,
            state_hash: block.hash.to_string(),
            transactions: block.body().diff().clone().try_into()?,
        })
    }
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLProtocolState {
    pub previous_state_hash: String,
    pub blockchain_state: GraphQLBlockchainState,
    pub consensus_state: GraphQLConsensusState,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLBlockchainState {
    pub snarked_ledger_hash: String,
    pub staged_ledger_hash: String,
    pub date: String,
    pub utc_date: String,
    pub staged_ledger_proof_emitted: bool,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLConsensusState {
    pub block_height: String,
    pub slot_since_genesis: String,
    pub slot: String,
    pub next_epoch_data: GraphQLEpochData,
    pub staking_epoch_data: GraphQLEpochData,
    pub epoch_count: String,
    pub min_window_density: String,
    pub total_currency: String,
    pub epoch: String,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLEpochData {
    pub ledger: GraphQLLedger,
    pub seed: String,
    pub start_checkpoint: String,
    pub lock_checkpoint: String,
    pub epoch_length: String,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLLedger {
    pub hash: String,
    pub total_currency: String,
}

impl TryFrom<mina_p2p_messages::v2::StagedLedgerDiffDiffDiffStableV2> for GraphQLTransactions {
    type Error = ConversionError;
    fn try_from(
        value: mina_p2p_messages::v2::StagedLedgerDiffDiffDiffStableV2,
    ) -> Result<Self, Self::Error> {
        use mina_p2p_messages::v2::{
            MinaBaseTransactionStatusStableV2, MinaBaseUserCommandStableV2,
        };

        let also_zkapp_commands = value
            .1
            .map_or_else(Vec::new, |v| v.commands.into_iter().collect::<Vec<_>>());

        let zkapp_commands = value
            .0
            .commands
            .into_iter()
            .chain(also_zkapp_commands)
            .rev()
            .map(|cmd| {
                // std::fs::create_dir_all("zkapps").unwrap();
                // let zkapp_path = format!("zkapps/{}", zkapp.hash().unwrap());
                // let path = PathBuf::from(zkapp_path.clone());
                // if !path.exists() {
                //     let mut buff = Vec::new();
                //     zkapp.binprot_write(&mut buff).unwrap();
                //     std::fs::write(zkapp_path, buff).unwrap();
                // }
                if let MinaBaseUserCommandStableV2::ZkappCommand(zkapp) = cmd.data {
                    let failure_reason =
                        if let MinaBaseTransactionStatusStableV2::Failed(failure_collection) =
                            cmd.status
                        {
                            let res = failure_collection
                                .0
                                .into_iter()
                                .enumerate()
                                .skip(1)
                                .map(|(index, failure_list)| {
                                    let fl =
                                        failure_list.into_iter().map(|v| v.to_string()).collect();
                                    GraphQLFailureReason {
                                        index: index.to_string(),
                                        failures: fl,
                                    }
                                })
                                .rev()
                                .collect();
                            Some(res)
                        } else {
                            None
                        };
                    let account_updates = zkapp
                        .account_updates
                        .clone()
                        .into_iter()
                        .map(|v| v.elt.account_update.try_into())
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Some(GraphQLZkapp {
                        hash: zkapp.hash()?.to_string(),
                        failure_reason,
                        id: zkapp.to_base64()?,
                        zkapp_command: GraphQLZkappCommand {
                            memo: zkapp.memo.to_base58check(),
                            account_updates,
                            fee_payer: GraphQLFeePayer::from(zkapp.fee_payer),
                        },
                    }))
                } else {
                    Ok(None)
                }
            })
            .collect::<Result<Vec<_>, Self::Error>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        Ok(Self { zkapp_commands })
    }
}

impl From<mina_p2p_messages::v2::MinaBaseEpochLedgerValueStableV1> for GraphQLLedger {
    fn from(value: mina_p2p_messages::v2::MinaBaseEpochLedgerValueStableV1) -> Self {
        Self {
            hash: value.hash.to_string(),
            total_currency: value.total_currency.as_u64().to_string(),
        }
    }
}

impl From<mina_p2p_messages::v2::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1>
    for GraphQLEpochData
{
    fn from(
        value: mina_p2p_messages::v2::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ) -> Self {
        Self {
            ledger: value.ledger.into(),
            seed: value.seed.to_string(),
            start_checkpoint: value.start_checkpoint.to_string(),
            lock_checkpoint: value.lock_checkpoint.to_string(),
            epoch_length: value.epoch_length.as_u32().to_string(),
        }
    }
}

impl
    From<
        mina_p2p_messages::v2::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    > for GraphQLEpochData
{
    fn from(
        value: mina_p2p_messages::v2::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    ) -> Self {
        Self {
            ledger: value.ledger.into(),
            seed: value.seed.to_string(),
            start_checkpoint: value.start_checkpoint.to_string(),
            lock_checkpoint: value.lock_checkpoint.to_string(),
            epoch_length: value.epoch_length.as_u32().to_string(),
        }
    }
}

impl From<mina_p2p_messages::v2::ConsensusProofOfStakeDataConsensusStateValueStableV2>
    for GraphQLConsensusState
{
    fn from(
        value: mina_p2p_messages::v2::ConsensusProofOfStakeDataConsensusStateValueStableV2,
    ) -> Self {
        let slot = value.curr_global_slot_since_hard_fork.slot_number.as_u32()
            % value
                .curr_global_slot_since_hard_fork
                .slots_per_epoch
                .as_u32();

        Self {
            block_height: value.blockchain_length.as_u32().to_string(),
            slot_since_genesis: value.global_slot_since_genesis.as_u32().to_string(),
            slot: slot.to_string(),
            next_epoch_data: value.next_epoch_data.into(),
            staking_epoch_data: value.staking_epoch_data.into(),
            epoch_count: value.epoch_count.as_u32().to_string(),
            min_window_density: value.min_window_density.as_u32().to_string(),
            total_currency: value.total_currency.as_u64().to_string(),
            epoch: value.epoch_count.as_u32().to_string(),
        }
    }
}

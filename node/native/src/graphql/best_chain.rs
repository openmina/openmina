use std::iter;
use std::{fs, path::Path};

use juniper::GraphQLObject;
use ledger::{scan_state::transaction_logic::Memo, FpExt};
use mina_p2p_messages::binprot::BinProtWrite;
use mina_p2p_messages::v2::{
    MinaBaseVerificationKeyWireStableV1Base64, ReceiptChainHash, TokenIdKeyHash,
};
use openmina_core::{block::ArcBlockWithHash, transaction::Transaction};

// pub struct GraphQLBestChain(pub Vec<GraphQLBestChainBlock>);

// #[juniper::graphql_object]
// impl GraphQLBestChain {
//     fn best_chain(&self) -> &Vec<GraphQLBestChainBlock> {
//         &self.0
//     }
// }

#[derive(GraphQLObject)]
#[graphql(description = "A Mina block")]
pub struct GraphQLBestChainBlock {
    pub protocol_state: GraphQLProtocolState,
    pub state_hash: String,
    pub transactions: GraphQLTransactions,
}

#[derive(GraphQLObject)]
pub struct GraphQLTransactions {
    pub zkapp_commands: Vec<GraphQLZkappCommand>,
}

#[derive(GraphQLObject)]
pub struct GraphQLZkappCommand {
    pub hash: String,
    pub failure_reason: Option<Vec<GraphQLFailureReason>>,
    pub memo: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLFailureReason {
    pub index: String,
    pub failures: Vec<String>,
}

#[derive(GraphQLObject)]
pub struct GraphQLProtocolState {
    pub previous_state_hash: String,
    pub blockchain_state: GraphQLBlockchainState,
    pub consensus_state: GraphQLConsensusState,
    // pub genesis_state_hash: StateHash,
    // pub blockchain_state: MinaStateBlockchainStateValueStableV2,
    // pub consensus_state: ConsensusProofOfStakeDataConsensusStateValueStableV2,
    // pub constants: MinaBaseProtocolConstantsCheckedValueStableV1,
}

#[derive(GraphQLObject)]
pub struct GraphQLBlockchainState {
    pub snarked_ledger_hash: String,
    pub staged_ledger_hash: String,
    pub date: String,
    pub utc_date: String,
    pub staged_ledger_proof_emitted: bool,
}

#[derive(GraphQLObject)]
pub struct GraphQLConsensusState {
    pub block_height: String,
    pub slot_since_genesis: String,
    pub slot: String,
    pub next_epoch_data: GraphQLEpochData,
    pub staking_epoch_data: GraphQLEpochData,
    // pub staking_epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    // pub next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    pub epoch_count: String,
    pub min_window_density: String,
    pub total_currency: String,
    pub epoch: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLEpochData {
    pub ledger: GraphQLLedger,
    pub seed: String,
    pub start_checkpoint: String,
    pub lock_checkpoint: String,
    pub epoch_length: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLLedger {
    pub hash: String,
    pub total_currency: String,
}

impl From<mina_p2p_messages::v2::StagedLedgerDiffDiffDiffStableV2> for GraphQLTransactions {
    fn from(value: mina_p2p_messages::v2::StagedLedgerDiffDiffDiffStableV2) -> Self {
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
            .filter_map(|cmd| {
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
                    Some(GraphQLZkappCommand {
                        hash: zkapp.hash().unwrap().to_string(),
                        failure_reason,
                        memo: serde_json::to_string_pretty(&zkapp.memo)
                            .unwrap()
                            .trim_matches('"')
                            .to_string(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Self { zkapp_commands }
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

// impl From<mina_p2p_messages::v2::MinaStateBlockchainStateValueStableV2> for GraphQLBlockchainState {
//     fn from(value: mina_p2p_messages::v2::MinaStateBlockchainStateValueStableV2) -> Self {
//         Self {
//             snarked_ledger_hash: value.ledger_proof_statement.target.first_pass_ledger.to_string(),
//             staged_ledger_hash: value.staged_ledger_hash.non_snark.ledger_hash.to_string(),
//             date: value.timestamp.to_string(),
//             // TODO(adonagy): verify this
//             utc_date: value.timestamp.to_string(),
//             staged_ledger_proof_emitted: value.staged_ledger_hash.non_snark.
//         }
//     }
// }

// impl From<mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2> for GraphQLProtocolState {
//     fn from(value: mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2) -> Self {
//         value.
//         todo!()
//     }
// }

impl From<ArcBlockWithHash> for GraphQLBestChainBlock {
    fn from(value: ArcBlockWithHash) -> Self {
        let blockchain_state = GraphQLBlockchainState {
            snarked_ledger_hash: value.snarked_ledger_hash().to_string(),
            staged_ledger_hash: value
                .staged_ledger_hashes()
                .non_snark
                .ledger_hash
                .to_string(),
            date: value
                .header()
                .protocol_state
                .body
                .blockchain_state
                .timestamp
                .to_string(),
            utc_date: value
                .header()
                .protocol_state
                .body
                .blockchain_state
                .timestamp
                .to_string(),
            // staged_ledger_proof_emitted: value.body().has_emitted_proof(),
            // FIXME: info comming from Breadcrumb, which is not implemented
            staged_ledger_proof_emitted: false,
        };

        let protocol_state = GraphQLProtocolState {
            previous_state_hash: value.pred_hash().to_string(),
            blockchain_state,
            consensus_state: value
                .header()
                .protocol_state
                .body
                .consensus_state
                .clone()
                .into(),
        };

        Self {
            protocol_state,
            state_hash: value.hash.to_string(),
            transactions: value.body().diff().clone().into(),
        }
    }
}

// impl From<Vec<ArcBlockWithHash>> for GraphQLBestChain {
//     fn from(value: Vec<ArcBlockWithHash>) -> Self {
//         GraphQLBestChain(value.into_iter().map(|b| b.into()).collect())
//     }
// }

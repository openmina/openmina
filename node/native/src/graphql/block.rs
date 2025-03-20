use crate::graphql::{
    account::GraphQLAccount,
    zkapp::{GraphQLFailureReason, GraphQLFeePayer, GraphQLZkappCommand},
};
use juniper::{graphql_object, FieldResult, GraphQLEnum, GraphQLObject};
use ledger::AccountId;
use mina_p2p_messages::v2::{
    MinaBaseSignedCommandPayloadBodyStableV2, MinaBaseSignedCommandStableV2,
    MinaBaseStakeDelegationStableV2, TransactionSnarkWorkTStableV2,
};
use mina_signer::CompressedPubKey;
use node::account::AccountPublicKey;
use openmina_core::block::AppliedBlock;

use super::{zkapp::GraphQLZkapp, Context, ConversionError};

#[derive(Debug)]
/// Location [src/lib/mina_graphql/types.ml:2095](https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_graphql/types.ml#L2095-L2151)
pub(crate) struct GraphQLBlock {
    creator: String,
    creator_account_key: CompressedPubKey,
    winner_account_key: CompressedPubKey,
    state_hash: String,
    /// Experimental: Bigint field-element representation of stateHash
    state_hash_field: String,
    protocol_state: GraphQLProtocolState,
    /// Public key of account that produced this block
    /// use creatorAccount field instead
    transactions: GraphQLTransactions,
    /// Base58Check-encoded hash of the state after this block
    /// Count of user command transactions in the block
    command_transaction_count: i32,
    snark_jobs: Vec<GraphQLSnarkJob>,
}

#[graphql_object(context = Context)]
#[graphql(description = "A Mina block")]
impl GraphQLBlock {
    fn creator(&self) -> &str {
        &self.creator
    }

    async fn creator_account(&self, context: &Context) -> FieldResult<Box<GraphQLAccount>> {
        let account_id = AccountId::new_with_default_token(self.creator_account_key.clone());
        let account_result = context
            .account_loader
            .try_load(account_id)
            .await
            .map_err(|e| {
                juniper::FieldError::new(
                    format!("Failed to load delegate account: {}", e),
                    juniper::Value::null(),
                )
            })?;

        // Handle the result
        match account_result {
            Ok(account) => Ok(Box::new(account)),
            Err(e) => Err(juniper::FieldError::new(
                format!("Error loading delegate account: {}", e),
                juniper::Value::null(),
            )),
        }
    }

    async fn winner_account(&self, context: &Context) -> FieldResult<Box<GraphQLAccount>> {
        let account_id = AccountId::new_with_default_token(self.winner_account_key.clone());
        let account_result = context
            .account_loader
            .try_load(account_id)
            .await
            .map_err(|e| {
                juniper::FieldError::new(
                    format!("Failed to load delegate account: {}", e),
                    juniper::Value::null(),
                )
            })?;

        // Handle the result
        match account_result {
            Ok(account) => Ok(Box::new(account)),
            Err(e) => Err(juniper::FieldError::new(
                format!("Error loading delegate account: {}", e),
                juniper::Value::null(),
            )),
        }
    }

    async fn state_hash(&self) -> &str {
        &self.state_hash
    }

    /// Experimental: Bigint field-element representation of stateHash
    async fn state_hash_field(&self) -> &str {
        &self.state_hash_field
    }

    async fn protocol_state(&self) -> &GraphQLProtocolState {
        &self.protocol_state
    }

    async fn transactions(&self) -> &GraphQLTransactions {
        &self.transactions
    }

    async fn command_transaction_count(&self) -> i32 {
        self.command_transaction_count
    }

    async fn snark_jobs(&self) -> &Vec<GraphQLSnarkJob> {
        &self.snark_jobs
    }
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLSnarkJob {
    pub fee: String,
    pub prover: String,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLTransactions {
    pub zkapp_commands: Vec<GraphQLZkapp>,
    pub user_commands: Vec<GraphQLUserCommands>,
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLUserCommands {
    pub amount: Option<String>,
    pub failure_reason: Option<String>,
    pub fee: String,
    pub fee_token: String,
    pub from: String,
    pub hash: String,
    pub id: String,
    pub is_delegation: bool,
    pub kind: GraphQLUserCommandsKind,
    pub memo: String,
    pub nonce: i32,
    pub to: String,
    pub token: String,
    pub valid_until: String,
}

#[derive(Clone, Copy, Debug, GraphQLEnum)]
#[allow(non_camel_case_types)]
pub enum GraphQLUserCommandsKind {
    PAYMENT,
    STAKE_DELEGATION,
}

impl TryFrom<AppliedBlock> for GraphQLBlock {
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

        let command_transaction_count = block.body().diff().0.commands.len() as i32;

        let snark_jobs = block
            .body()
            .completed_works_iter()
            .map(GraphQLSnarkJob::from)
            .collect();

        Ok(Self {
            creator_account_key: AccountPublicKey::from(block.producer().clone())
                .try_into()
                .map_err(|_| ConversionError::Custom("Invalid public key".to_string()))?,
            winner_account_key: AccountPublicKey::from(block.block_stake_winner().clone())
                .try_into()
                .map_err(|_| ConversionError::Custom("Invalid public key".to_string()))?,
            protocol_state,
            state_hash: block.hash.to_string(),
            state_hash_field: block.hash.to_decimal(),
            creator: block.producer().to_string(),
            transactions: block.body().diff().clone().try_into()?,
            command_transaction_count,
            snark_jobs,
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

        let commands = value
            .0
            .commands
            .into_iter()
            .chain(also_zkapp_commands)
            .rev();

        let mut zkapp_commands = Vec::new();
        let mut user_commands = Vec::new();

        for command in commands {
            match command.data {
                MinaBaseUserCommandStableV2::SignedCommand(user_command) => {
                    user_commands.push(GraphQLUserCommands::try_from(user_command)?);
                }
                MinaBaseUserCommandStableV2::ZkappCommand(zkapp) => {
                    let failure_reason =
                        if let MinaBaseTransactionStatusStableV2::Failed(failure_collection) =
                            command.status
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

                    zkapp_commands.push(GraphQLZkapp {
                        hash: zkapp.hash()?.to_string(),
                        failure_reason,
                        id: zkapp.to_base64()?,
                        zkapp_command: GraphQLZkappCommand {
                            memo: zkapp.memo.to_base58check(),
                            account_updates,
                            fee_payer: GraphQLFeePayer::from(zkapp.fee_payer),
                        },
                    });
                }
            }
        }

        Ok(Self {
            zkapp_commands,
            user_commands,
        })
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

impl From<&TransactionSnarkWorkTStableV2> for GraphQLSnarkJob {
    fn from(value: &TransactionSnarkWorkTStableV2) -> Self {
        Self {
            fee: value.fee.to_string(),
            prover: value.prover.to_string(),
        }
    }
}

impl TryFrom<MinaBaseSignedCommandStableV2> for GraphQLUserCommands {
    type Error = ConversionError;

    fn try_from(user_command: MinaBaseSignedCommandStableV2) -> Result<Self, Self::Error> {
        let is_delegation = matches!(
            user_command.payload.body,
            MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(_)
        );
        let hash = user_command.hash()?.to_string();
        let id = user_command.to_base64()?;

        let fee = user_command.payload.common.fee.to_string();
        let memo = user_command.payload.common.memo.to_base58check();
        let nonce = user_command.payload.common.nonce.as_u32() as i32;
        let valid_until = user_command.payload.common.valid_until.as_u32().to_string();

        let (to, amount, kind) = match user_command.payload.body {
            MinaBaseSignedCommandPayloadBodyStableV2::Payment(payment) => (
                payment.receiver_pk.to_string(),
                Some(payment.amount.to_string()),
                GraphQLUserCommandsKind::PAYMENT,
            ),
            MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(
                MinaBaseStakeDelegationStableV2::SetDelegate { new_delegate },
            ) => (
                new_delegate.to_string(),
                None,
                GraphQLUserCommandsKind::STAKE_DELEGATION,
            ),
        };

        Ok(GraphQLUserCommands {
            hash,
            from: user_command.signer.to_string(),
            to,
            is_delegation,
            amount,
            failure_reason: Default::default(),
            fee,
            fee_token: Default::default(),
            id,
            kind,
            memo,
            nonce,
            token: Default::default(),
            valid_until,
        })
    }
}

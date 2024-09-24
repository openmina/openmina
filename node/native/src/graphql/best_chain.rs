use std::iter;
use std::str::FromStr;

use juniper::{GraphQLInputObject, GraphQLObject};
use ledger::{scan_state::transaction_logic::Memo, FpExt};
use mina_p2p_messages::b58::FromBase58Check;
use mina_p2p_messages::bigint::BigInt;
use mina_p2p_messages::list::List;
use mina_p2p_messages::pseq::PaddedSeq;
use mina_p2p_messages::string::{TokenSymbol, ZkAppUri};
use mina_p2p_messages::v2::{
    CurrencyAmountStableV1, CurrencyBalanceStableV1, CurrencyFeeStableV1,
    MinaBaseAccountUpdateAccountPreconditionStableV1,
    MinaBaseAccountUpdateAuthorizationKindStableV1, MinaBaseAccountUpdateBodyEventsStableV1,
    MinaBaseAccountUpdateBodyFeePayerStableV1, MinaBaseAccountUpdateBodyStableV1,
    MinaBaseAccountUpdateFeePayerStableV1, MinaBaseAccountUpdateMayUseTokenStableV1,
    MinaBaseAccountUpdatePreconditionsStableV1, MinaBaseAccountUpdateTStableV1,
    MinaBaseAccountUpdateUpdateStableV1, MinaBaseAccountUpdateUpdateStableV1AppStateA,
    MinaBaseAccountUpdateUpdateStableV1Delegate, MinaBaseAccountUpdateUpdateStableV1Permissions,
    MinaBaseAccountUpdateUpdateStableV1Timing, MinaBaseAccountUpdateUpdateStableV1TokenSymbol,
    MinaBaseAccountUpdateUpdateStableV1VerificationKey,
    MinaBaseAccountUpdateUpdateStableV1VotingFor, MinaBaseAccountUpdateUpdateStableV1ZkappUri,
    MinaBaseAccountUpdateUpdateTimingInfoStableV1, MinaBaseControlStableV2,
    MinaBasePermissionsStableV2, MinaBaseReceiptChainHashStableV1,
    MinaBaseSignedCommandMemoStableV1, MinaBaseUserCommandStableV2,
    MinaBaseVerificationKeyWireStableV1, MinaBaseVerificationKeyWireStableV1Base64,
    MinaBaseZkappCommandTStableV1WireStableV1,
    MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA,
    MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA,
    MinaBaseZkappCommandTStableV1WireStableV1Base64, MinaBaseZkappPreconditionAccountStableV2,
    MinaBaseZkappPreconditionAccountStableV2Balance,
    MinaBaseZkappPreconditionAccountStableV2BalanceA,
    MinaBaseZkappPreconditionAccountStableV2Delegate,
    MinaBaseZkappPreconditionAccountStableV2ProvedState,
    MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash,
    MinaBaseZkappPreconditionAccountStableV2StateA,
    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger,
    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed,
    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint,
    MinaBaseZkappPreconditionProtocolStateStableV1,
    MinaBaseZkappPreconditionProtocolStateStableV1Amount,
    MinaBaseZkappPreconditionProtocolStateStableV1AmountA,
    MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot,
    MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA,
    MinaBaseZkappPreconditionProtocolStateStableV1Length,
    MinaBaseZkappPreconditionProtocolStateStableV1LengthA,
    MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash,
    MinaNumbersGlobalSlotSinceGenesisMStableV1, MinaNumbersGlobalSlotSpanStableV1,
    MinaStateBlockchainStateValueStableV2SignedAmount, NonZeroCurvePoint, ReceiptChainHash,
    StateHash, TokenIdKeyHash,
};

use mina_signer::CompressedPubKey;
use node::account::AccountPublicKey;
use openmina_core::{block::ArcBlockWithHash, transaction::Transaction};

use super::account::{GraphQLAccount, GraphQLTiming, InputGraphQLTiming};

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
    pub zkapp_commands: Vec<GraphQLZkapp>,
}

#[derive(GraphQLObject)]
pub struct GraphQLZkapp {
    pub hash: String,
    pub failure_reason: Option<Vec<GraphQLFailureReason>>,
    /// Zkapp represented as base64 string
    pub id: String,
    pub zkapp_command: GraphQLZkappCommand,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLZkapp {
    // pub hash: String,
    // pub failure_reason: Option<Vec<GraphQLFailureReason>>,
    /// Zkapp represented as base64 string
    // pub id: String,
    pub zkapp_command: InputGraphQLZkappCommand,
}

#[derive(GraphQLObject)]
pub struct GraphQLZkappCommand {
    pub memo: String,
    pub account_updates: Vec<GraphQLAccountUpdate>,
    pub fee_payer: GraphQLFeePayer,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLZkappCommand {
    pub memo: String,
    pub account_updates: Vec<InputGraphQLAccountUpdate>,
    pub fee_payer: InputGraphQLFeePayer,
}

/// TODO(adonagy): Look into the handling of failures, this only returns successful zkapp commands
impl From<MinaBaseUserCommandStableV2> for GraphQLZkapp {
    fn from(value: MinaBaseUserCommandStableV2) -> Self {
        if let MinaBaseUserCommandStableV2::ZkappCommand(zkapp) = value {
            let account_updates = zkapp
                .account_updates
                .clone()
                .into_iter()
                .map(|v| v.elt.account_update.into())
                .collect();
            GraphQLZkapp {
                hash: zkapp.hash().unwrap().to_string(),
                failure_reason: None,
                id: zkapp.to_base64().unwrap(),
                zkapp_command: GraphQLZkappCommand {
                    memo: serde_json::to_string_pretty(&zkapp.memo)
                        .unwrap()
                        .trim_matches('"')
                        .to_string(),
                    account_updates,
                    fee_payer: GraphQLFeePayer::from(zkapp.fee_payer),
                },
            }
        } else {
            unreachable!()
        }
    }
}

impl From<InputGraphQLZkappCommand> for MinaBaseUserCommandStableV2 {
    fn from(value: InputGraphQLZkappCommand) -> Self {
        MinaBaseUserCommandStableV2::ZkappCommand(MinaBaseZkappCommandTStableV1WireStableV1 {
            fee_payer: value.fee_payer.into(),
            account_updates: value
                .account_updates
                .into_iter()
                .map(
                    |update| MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA {
                        elt: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
                            account_update: update.into(),
                            account_update_digest: (),
                            // TODO: look into this, in the body of the account update there are fields callData and callDepth, is it related?
                            calls: List::new(),
                        },
                        stack_hash: (),
                    },
                )
                .collect(),
            memo: MinaBaseSignedCommandMemoStableV1::from_base58check(&value.memo).unwrap(),
        })
    }
}

#[derive(GraphQLObject)]
pub struct GraphQLFeePayer {
    pub body: GraphQLFeePayerBody,
    pub authorization: String,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLFeePayer {
    pub body: InputGraphQLFeePayerBody,
    pub authorization: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLFeePayerBody {
    pub public_key: String,
    pub fee: String,
    pub valid_until: Option<String>,
    pub nonce: String,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLFeePayerBody {
    pub public_key: String,
    pub fee: String,
    pub valid_until: Option<String>,
    pub nonce: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLAccountUpdate {
    pub body: GraphQLAccountUpdateBody,
    pub authorization: GraphQLAuthorization,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLAccountUpdate {
    pub body: InputGraphQLAccountUpdateBody,
    pub authorization: InputGraphQLAuthorization,
}

#[derive(GraphQLObject)]
pub struct GraphQLAuthorization {
    pub proof: Option<String>,
    pub signature: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLAuthorization {
    pub proof: Option<String>,
    pub signature: Option<String>,
}

impl From<MinaBaseControlStableV2> for GraphQLAuthorization {
    fn from(value: MinaBaseControlStableV2) -> Self {
        match value {
            MinaBaseControlStableV2::Signature(signature) => GraphQLAuthorization {
                proof: None,
                signature: Some(signature.to_string()),
            },
            MinaBaseControlStableV2::Proof(proof) => GraphQLAuthorization {
                proof: Some(
                    serde_json::to_string_pretty(&proof)
                        .unwrap()
                        .trim_matches('"')
                        .to_string(),
                ),
                signature: None,
            },
            MinaBaseControlStableV2::NoneGiven => GraphQLAuthorization {
                proof: None,
                signature: None,
            },
        }
    }
}

impl From<InputGraphQLAuthorization> for MinaBaseControlStableV2 {
    fn from(value: InputGraphQLAuthorization) -> Self {
        if let Some(signature) = value.signature {
            MinaBaseControlStableV2::Signature(signature.parse().unwrap())
        } else if let Some(proof) = value.proof {
            MinaBaseControlStableV2::Proof(serde_json::from_str(&proof).unwrap())
        } else {
            MinaBaseControlStableV2::NoneGiven
        }
    }
}

#[derive(GraphQLObject)]
pub struct GraphQLAccountUpdateBody {
    pub public_key: String,
    pub token_id: String,
    pub use_full_commitment: bool,
    pub increment_nonce: bool,
    pub update: GraphQLAccountUpdateUpdate,
    pub balance_change: GraphQLBalanceChange,
    pub events: Vec<Vec<String>>,
    pub actions: Vec<Vec<String>>,
    pub call_data: String,
    pub call_depth: i32,
    pub preconditions: GraphQLPreconditions,
    pub may_use_token: GraphQLMayUseToken,
    pub authorization_kind: GraphQLAuthorizationKind,
    pub implicit_account_creation_fee: bool,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLAccountUpdateBody {
    pub public_key: String,
    pub token_id: String,
    pub use_full_commitment: bool,
    pub increment_nonce: bool,
    pub update: InputGraphQLAccountUpdateUpdate,
    pub balance_change: InputGraphQLBalanceChange,
    pub events: Vec<Vec<String>>,
    pub actions: Vec<Vec<String>>,
    pub call_data: String,
    pub call_depth: i32,
    pub preconditions: InputGraphQLPreconditions,
    pub may_use_token: InputGraphQLMayUseToken,
    pub authorization_kind: InputGraphQLAuthorizationKind,
    pub implicit_account_creation_fee: bool,
}

#[derive(GraphQLObject)]
pub struct GraphQLAuthorizationKind {
    pub is_signed: bool,
    pub is_proved: bool,
    pub verification_key_hash: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLAuthorizationKind {
    pub is_signed: bool,
    pub is_proved: bool,
    pub verification_key_hash: Option<String>,
}

impl From<MinaBaseAccountUpdateAuthorizationKindStableV1> for GraphQLAuthorizationKind {
    fn from(value: MinaBaseAccountUpdateAuthorizationKindStableV1) -> Self {
        match value {
            MinaBaseAccountUpdateAuthorizationKindStableV1::Signature => GraphQLAuthorizationKind {
                is_signed: true,
                is_proved: false,
                verification_key_hash: None,
            },
            MinaBaseAccountUpdateAuthorizationKindStableV1::Proof(proof) => {
                GraphQLAuthorizationKind {
                    is_signed: false,
                    is_proved: true,
                    verification_key_hash: Some(proof.to_fp().unwrap().to_decimal()),
                }
            }
            MinaBaseAccountUpdateAuthorizationKindStableV1::NoneGiven => GraphQLAuthorizationKind {
                is_signed: false,
                is_proved: false,
                verification_key_hash: None,
            },
        }
    }
}

impl From<InputGraphQLAuthorizationKind> for MinaBaseAccountUpdateAuthorizationKindStableV1 {
    fn from(value: InputGraphQLAuthorizationKind) -> Self {
        if value.is_signed {
            MinaBaseAccountUpdateAuthorizationKindStableV1::Signature
        } else if value.is_proved {
            MinaBaseAccountUpdateAuthorizationKindStableV1::Proof(
                BigInt::from_decimal(&value.verification_key_hash.unwrap()).unwrap(),
            )
        } else {
            MinaBaseAccountUpdateAuthorizationKindStableV1::NoneGiven
        }
    }
}

#[derive(GraphQLObject)]
pub struct GraphQLMayUseToken {
    pub parents_own_token: bool,
    pub inherit_from_parent: bool,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLMayUseToken {
    pub parents_own_token: bool,
    pub inherit_from_parent: bool,
}

impl From<MinaBaseAccountUpdateMayUseTokenStableV1> for GraphQLMayUseToken {
    fn from(value: MinaBaseAccountUpdateMayUseTokenStableV1) -> Self {
        match value {
            MinaBaseAccountUpdateMayUseTokenStableV1::ParentsOwnToken => GraphQLMayUseToken {
                parents_own_token: true,
                inherit_from_parent: false,
            },
            MinaBaseAccountUpdateMayUseTokenStableV1::InheritFromParent => GraphQLMayUseToken {
                parents_own_token: false,
                inherit_from_parent: true,
            },
            MinaBaseAccountUpdateMayUseTokenStableV1::No => GraphQLMayUseToken {
                parents_own_token: false,
                inherit_from_parent: false,
            },
        }
    }
}

impl From<InputGraphQLMayUseToken> for MinaBaseAccountUpdateMayUseTokenStableV1 {
    fn from(value: InputGraphQLMayUseToken) -> Self {
        if value.parents_own_token {
            MinaBaseAccountUpdateMayUseTokenStableV1::ParentsOwnToken
        } else if value.inherit_from_parent {
            MinaBaseAccountUpdateMayUseTokenStableV1::InheritFromParent
        } else {
            MinaBaseAccountUpdateMayUseTokenStableV1::No
        }
    }
}

#[derive(GraphQLObject)]
pub struct GraphQLEvent {
    pub event: String,
    pub data: String,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLEvent {
    pub event: String,
    pub data: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLAction {
    pub action: String,
    pub data: String,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLAction {
    pub action: String,
    pub data: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLPreconditions {
    pub network: GraphQLPreconditionsNetwork,
    pub account: GraphQLPreconditionsAccount,
    pub valid_while: Option<GraphQLPreconditionsNetworkBounds>,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLPreconditions {
    pub network: InputGraphQLPreconditionsNetwork,
    pub account: InputGraphQLPreconditionsAccount,
    pub valid_while: Option<InputGraphQLPreconditionsNetworkBounds>,
}

#[derive(GraphQLObject)]
pub struct GraphQLPreconditionsAccount {
    pub balance: Option<GraphQLPreconditionsNetworkBounds>,
    pub nonce: Option<GraphQLPreconditionsNetworkBounds>,
    pub receipt_chain_hash: Option<String>,
    pub delegate: Option<String>,
    pub state: Vec<Option<String>>,
    pub action_state: Option<String>,
    pub proved_state: Option<String>,
    pub is_new: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLPreconditionsAccount {
    pub balance: Option<InputGraphQLPreconditionsNetworkBounds>,
    pub nonce: Option<InputGraphQLPreconditionsNetworkBounds>,
    pub receipt_chain_hash: Option<String>,
    pub delegate: Option<String>,
    pub state: Vec<Option<String>>,
    pub action_state: Option<String>,
    pub proved_state: Option<String>,
    pub is_new: Option<String>,
}

impl From<MinaBaseAccountUpdateAccountPreconditionStableV1> for GraphQLPreconditionsAccount {
    fn from(value: MinaBaseAccountUpdateAccountPreconditionStableV1) -> Self {
        Self {
            balance: if let MinaBaseZkappPreconditionAccountStableV2Balance::Check(v) =
                value.0.balance
            {
                Some(GraphQLPreconditionsNetworkBounds {
                    upper: v.upper.as_u64() as i32,
                    lower: v.lower.as_u64() as i32,
                })
            } else {
                None
            },
            nonce: if let MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(v) =
                value.0.nonce
            {
                Some(GraphQLPreconditionsNetworkBounds {
                    upper: v.upper.as_u32() as i32,
                    lower: v.lower.as_u32() as i32,
                })
            } else {
                None
            },
            receipt_chain_hash:
                if let MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash::Check(v) =
                    value.0.receipt_chain_hash
                {
                    Some(v.to_fp().unwrap().to_decimal())
                } else {
                    None
                },
            delegate: if let MinaBaseZkappPreconditionAccountStableV2Delegate::Check(v) =
                value.0.delegate
            {
                Some(v.to_string())
            } else {
                None
            },
            state: value
                .0
                .state
                .clone()
                .iter()
                .map(|v| {
                    if let MinaBaseZkappPreconditionAccountStableV2StateA::Check(state_value) = v {
                        Some(state_value.to_fp().unwrap().to_decimal())
                    } else {
                        None
                    }
                })
                .collect(),
            action_state: if let MinaBaseZkappPreconditionAccountStableV2StateA::Check(value) =
                value.0.action_state
            {
                Some(value.to_fp().unwrap().to_decimal())
            } else {
                None
            },
            proved_state: if let MinaBaseZkappPreconditionAccountStableV2ProvedState::Check(v) =
                value.0.proved_state
            {
                Some(v.to_string())
            } else {
                None
            },
            is_new: if let MinaBaseZkappPreconditionAccountStableV2ProvedState::Check(v) =
                value.0.is_new
            {
                Some(v.to_string())
            } else {
                None
            },
        }
    }
}

impl From<InputGraphQLPreconditionsAccount> for MinaBaseAccountUpdateAccountPreconditionStableV1 {
    fn from(value: InputGraphQLPreconditionsAccount) -> Self {
        let state: Vec<_> = value
            .state
            .iter()
            .map(|v| {
                if let Some(state) = v {
                    MinaBaseZkappPreconditionAccountStableV2StateA::Check(
                        BigInt::from_decimal(&state).unwrap(),
                    )
                } else {
                    MinaBaseZkappPreconditionAccountStableV2StateA::Ignore
                }
            })
            .collect();
        Self(MinaBaseZkappPreconditionAccountStableV2 {
            balance: if let Some(balance) = value.balance {
                MinaBaseZkappPreconditionAccountStableV2Balance::Check(
                    MinaBaseZkappPreconditionAccountStableV2BalanceA {
                        lower: CurrencyBalanceStableV1(CurrencyAmountStableV1(
                            (balance.lower as u64).into(),
                        )),
                        upper: CurrencyBalanceStableV1(CurrencyAmountStableV1(
                            (balance.upper as u64).into(),
                        )),
                    },
                )
            } else {
                MinaBaseZkappPreconditionAccountStableV2Balance::Ignore
            },
            nonce: if let Some(nonce) = value.nonce {
                MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                        lower: (nonce.lower as u32).into(),
                        upper: (nonce.upper as u32).into(),
                    },
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1Length::Ignore
            },
            receipt_chain_hash: if let Some(receipt_chain_hash) = value.receipt_chain_hash {
                MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash::Check(
                    MinaBaseReceiptChainHashStableV1(
                        BigInt::from_decimal(&receipt_chain_hash).unwrap(),
                    ),
                )
            } else {
                MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash::Ignore
            },
            delegate: if let Some(delegate) = value.delegate {
                MinaBaseZkappPreconditionAccountStableV2Delegate::Check(
                    AccountPublicKey::from_str(&delegate).unwrap().into(),
                )
            } else {
                MinaBaseZkappPreconditionAccountStableV2Delegate::Ignore
            },
            state: PaddedSeq(state.try_into().unwrap()),
            action_state: if let Some(action_state) = value.action_state {
                MinaBaseZkappPreconditionAccountStableV2StateA::Check(
                    BigInt::from_decimal(&action_state).unwrap(),
                )
            } else {
                MinaBaseZkappPreconditionAccountStableV2StateA::Ignore
            },
            proved_state: if let Some(proved_state) = value.proved_state {
                MinaBaseZkappPreconditionAccountStableV2ProvedState::Check(
                    proved_state.parse().unwrap(),
                )
            } else {
                MinaBaseZkappPreconditionAccountStableV2ProvedState::Ignore
            },
            is_new: if let Some(is_new) = value.is_new {
                MinaBaseZkappPreconditionAccountStableV2ProvedState::Check(is_new.parse().unwrap())
            } else {
                MinaBaseZkappPreconditionAccountStableV2ProvedState::Ignore
            },
        })
    }
}
#[derive(GraphQLObject)]
pub struct GraphQLPreconditionsNetwork {
    pub snarked_ledger_hash: Option<String>,
    pub blockchain_length: Option<GraphQLPreconditionsNetworkBounds>,
    pub min_window_density: Option<GraphQLPreconditionsNetworkBounds>,
    pub total_currency: Option<GraphQLPreconditionsNetworkBounds>,
    pub global_slot_since_genesis: Option<GraphQLPreconditionsNetworkBounds>,
    pub staking_epoch_data: GraphQLPreconditionsNetworkEpochData,
    pub next_epoch_data: GraphQLPreconditionsNetworkEpochData,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLPreconditionsNetwork {
    pub snarked_ledger_hash: Option<String>,
    pub blockchain_length: Option<InputGraphQLPreconditionsNetworkBounds>,
    pub min_window_density: Option<InputGraphQLPreconditionsNetworkBounds>,
    pub total_currency: Option<InputGraphQLPreconditionsNetworkBounds>,
    pub global_slot_since_genesis: Option<InputGraphQLPreconditionsNetworkBounds>,
    pub staking_epoch_data: InputGraphQLPreconditionsNetworkEpochData,
    pub next_epoch_data: InputGraphQLPreconditionsNetworkEpochData,
}

#[derive(GraphQLObject)]
pub struct GraphQLPreconditionsNetworkEpochData {
    pub ledger: GraphQLPreconditionsNetworkLedger,
    pub seed: Option<String>,
    pub start_checkpoint: Option<String>,
    pub lock_checkpoint: Option<String>,
    pub epoch_length: Option<GraphQLPreconditionsNetworkBounds>,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLPreconditionsNetworkEpochData {
    pub ledger: InputGraphQLPreconditionsNetworkLedger,
    pub seed: Option<String>,
    pub start_checkpoint: Option<String>,
    pub lock_checkpoint: Option<String>,
    pub epoch_length: Option<InputGraphQLPreconditionsNetworkBounds>,
}

#[derive(GraphQLObject)]
pub struct GraphQLPreconditionsNetworkLedger {
    pub hash: Option<String>,
    pub total_currency: Option<GraphQLPreconditionsNetworkBounds>,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLPreconditionsNetworkLedger {
    pub hash: Option<String>,
    pub total_currency: Option<InputGraphQLPreconditionsNetworkBounds>,
}
#[derive(GraphQLObject)]
pub struct GraphQLPreconditionsNetworkBounds {
    pub upper: i32,
    pub lower: i32,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLPreconditionsNetworkBounds {
    pub upper: i32,
    pub lower: i32,
}

impl From<MinaBaseZkappPreconditionProtocolStateEpochDataStableV1>
    for GraphQLPreconditionsNetworkEpochData
{
    fn from(value: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1) -> Self {
        Self {
            ledger: GraphQLPreconditionsNetworkLedger::from(value.ledger),
            seed: if let MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed::Check(v) = value.seed {
                Some(v.to_string())
            } else {
                None
            },
            start_checkpoint: if let MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Check(v) = value.start_checkpoint {
                Some(v.to_string())
            } else {
                None
            },
            lock_checkpoint: if let MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Check(v) = value.lock_checkpoint {
                Some(v.to_string())
            } else {
                None
            },
            epoch_length: if let MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(v) = value.epoch_length {
                Some(GraphQLPreconditionsNetworkBounds {
                    upper: v.upper.as_u32() as i32,
                    lower: v.lower.as_u32() as i32,
                })
            } else {
                None
            },
        }
    }
}

impl From<InputGraphQLPreconditionsNetworkEpochData>
    for MinaBaseZkappPreconditionProtocolStateEpochDataStableV1
{
    fn from(value: InputGraphQLPreconditionsNetworkEpochData) -> Self {
        Self {
            ledger: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger::from(
                value.ledger,
            ),
            seed: if let Some(seed) = value.seed {
                MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed::Check(
                    seed.parse().unwrap(),
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed::Ignore
            },
            start_checkpoint: if let Some(start_checkpoint) = value.start_checkpoint {
                MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Check(
                    start_checkpoint.parse().unwrap(),
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Ignore
            },
            lock_checkpoint: if let Some(lock_checkpoint) = value.lock_checkpoint {
                MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Check(
                    lock_checkpoint.parse().unwrap(),
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Ignore
            },
            epoch_length: if let Some(epoch_length) = value.epoch_length {
                MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                        lower: (epoch_length.lower as u32).into(),
                        upper: (epoch_length.upper as u32).into(),
                    },
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1Length::Ignore
            },
        }
    }
}
impl From<MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger>
    for GraphQLPreconditionsNetworkLedger
{
    fn from(value: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger) -> Self {
        Self {
            hash: if let MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Check(v) =
                value.hash
            {
                Some(v.to_string())
            } else {
                None
            },
            total_currency: if let MinaBaseZkappPreconditionProtocolStateStableV1Amount::Check(v) =
                value.total_currency
            {
                Some(GraphQLPreconditionsNetworkBounds {
                    upper: v.upper.as_u64() as i32,
                    lower: v.lower.as_u64() as i32,
                })
            } else {
                None
            },
        }
    }
}

impl From<InputGraphQLPreconditionsNetworkLedger>
    for MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger
{
    fn from(value: InputGraphQLPreconditionsNetworkLedger) -> Self {
        Self {
            hash: if let Some(hash) = value.hash {
                MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Check(
                    hash.parse().unwrap(),
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Ignore
            },
            total_currency: if let Some(total_currency) = value.total_currency {
                MinaBaseZkappPreconditionProtocolStateStableV1Amount::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
                        lower: CurrencyAmountStableV1((total_currency.lower as u64).into()),
                        upper: CurrencyAmountStableV1((total_currency.upper as u64).into()),
                    },
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1Amount::Ignore
            },
        }
    }
}

impl From<MinaBaseZkappPreconditionProtocolStateStableV1> for GraphQLPreconditionsNetwork {
    fn from(value: MinaBaseZkappPreconditionProtocolStateStableV1) -> Self {
        Self {
            snarked_ledger_hash:
                if let MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Check(v) =
                    value.snarked_ledger_hash
                {
                    Some(v.to_string())
                } else {
                    None
                },
            blockchain_length: if let MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                v,
            ) = value.blockchain_length
            {
                Some(GraphQLPreconditionsNetworkBounds {
                    upper: v.upper.as_u32() as i32,
                    lower: v.lower.as_u32() as i32,
                })
            } else {
                None
            },
            min_window_density: if let MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                v,
            ) = value.min_window_density
            {
                Some(GraphQLPreconditionsNetworkBounds {
                    upper: v.upper.as_u32() as i32,
                    lower: v.lower.as_u32() as i32,
                })
            } else {
                None
            },
            total_currency: if let MinaBaseZkappPreconditionProtocolStateStableV1Amount::Check(v) =
                value.total_currency
            {
                Some(GraphQLPreconditionsNetworkBounds {
                    upper: v.upper.as_u64() as i32,
                    lower: v.lower.as_u64() as i32,
                })
            } else {
                None
            },
            global_slot_since_genesis:
                if let MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Check(v) =
                    value.global_slot_since_genesis
                {
                    Some(GraphQLPreconditionsNetworkBounds {
                        upper: v.upper.as_u32() as i32,
                        lower: v.lower.as_u32() as i32,
                    })
                } else {
                    None
                },
            staking_epoch_data: GraphQLPreconditionsNetworkEpochData::from(
                value.staking_epoch_data,
            ),
            next_epoch_data: GraphQLPreconditionsNetworkEpochData::from(value.next_epoch_data),
        }
    }
}

impl From<InputGraphQLPreconditionsNetwork> for MinaBaseZkappPreconditionProtocolStateStableV1 {
    fn from(value: InputGraphQLPreconditionsNetwork) -> Self {
        Self {
            snarked_ledger_hash: if let Some(snarked_ledger_hash) = value.snarked_ledger_hash {
                MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Check(
                    snarked_ledger_hash.parse().unwrap(),
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Ignore
            },
            blockchain_length: if let Some(blockchain_length) = value.blockchain_length {
                MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                        lower: (blockchain_length.lower as u32).into(),
                        upper: (blockchain_length.upper as u32).into(),
                    },
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1Length::Ignore
            },
            min_window_density: if let Some(min_window_density) = value.min_window_density {
                MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                        lower: (min_window_density.lower as u32).into(),
                        upper: (min_window_density.upper as u32).into(),
                    },
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1Length::Ignore
            },
            total_currency: if let Some(total_currency) = value.total_currency {
                MinaBaseZkappPreconditionProtocolStateStableV1Amount::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
                        lower: CurrencyAmountStableV1((total_currency.lower as u64).into()),
                        upper: CurrencyAmountStableV1((total_currency.upper as u64).into()),
                    },
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1Amount::Ignore
            },
            global_slot_since_genesis: if let Some(global_slot_since_genesis) =
                value.global_slot_since_genesis
            {
                MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA {
                        lower: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
                            (global_slot_since_genesis.lower as u32).into(),
                        ),
                        upper: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
                            (global_slot_since_genesis.upper as u32).into(),
                        ),
                    },
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Ignore
            },
            staking_epoch_data: value.staking_epoch_data.into(),
            next_epoch_data: value.next_epoch_data.into(),
        }
    }
}

#[derive(GraphQLObject)]
pub struct GraphQLAccountUpdateUpdate {
    pub app_state: Vec<Option<String>>,
    pub delegate: Option<String>,
    pub verification_key: Option<String>,
    pub permissions: Option<GraphQLAccountUpdateUpdatePermissions>,
    pub zkapp_uri: Option<String>,
    pub token_symbol: Option<String>,
    pub timing: Option<GraphQLTiming>,
    pub voting_for: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLAccountUpdateUpdate {
    pub app_state: Vec<Option<String>>,
    pub delegate: Option<String>,
    pub verification_key: Option<String>,
    pub permissions: Option<InputGraphQLAccountUpdateUpdatePermissions>,
    pub zkapp_uri: Option<String>,
    pub token_symbol: Option<String>,
    pub timing: Option<InputGraphQLTiming>,
    pub voting_for: Option<String>,
}

#[derive(GraphQLObject)]
pub struct GraphQLAccountUpdateUpdatePermissions {
    pub edit_state: String,
    pub access: String,
    pub send: String,
    pub receive: String,
    pub set_delegate: String,
    pub set_permissions: String,
    pub set_verification_key: [String; 2],
    pub set_zkapp_uri: String,
    pub edit_action_state: String,
    pub set_token_symbol: String,
    pub set_timing: String,
    pub set_voting_for: String,
    pub increment_nonce: String,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLAccountUpdateUpdatePermissions {
    pub edit_state: String,
    pub access: String,
    pub send: String,
    pub receive: String,
    pub set_delegate: String,
    pub set_permissions: String,
    pub set_verification_key: [String; 2],
    pub set_zkapp_uri: String,
    pub edit_action_state: String,
    pub set_token_symbol: String,
    pub set_timing: String,
    pub set_voting_for: String,
    pub increment_nonce: String,
}

impl From<MinaBasePermissionsStableV2> for GraphQLAccountUpdateUpdatePermissions {
    fn from(value: MinaBasePermissionsStableV2) -> Self {
        Self {
            edit_state: value.edit_state.to_string(),
            access: value.access.to_string(),
            send: value.send.to_string(),
            receive: value.receive.to_string(),
            set_delegate: value.set_delegate.to_string(),
            set_permissions: value.set_permissions.to_string(),
            set_verification_key: [
                value.set_verification_key.0.to_string(),
                value.set_verification_key.1.to_string(),
            ],
            set_zkapp_uri: value.set_zkapp_uri.to_string(),
            edit_action_state: value.edit_action_state.to_string(),
            set_token_symbol: value.set_token_symbol.to_string(),
            set_timing: value.set_timing.to_string(),
            set_voting_for: value.set_voting_for.to_string(),
            increment_nonce: value.increment_nonce.to_string(),
        }
    }
}

impl From<InputGraphQLAccountUpdateUpdatePermissions> for MinaBasePermissionsStableV2 {
    fn from(value: InputGraphQLAccountUpdateUpdatePermissions) -> Self {
        Self {
            edit_state: value.edit_state.parse().unwrap(),
            access: value.access.parse().unwrap(),
            send: value.send.parse().unwrap(),
            receive: value.receive.parse().unwrap(),
            set_delegate: value.set_delegate.parse().unwrap(),
            set_permissions: value.set_permissions.parse().unwrap(),
            set_verification_key: (
                value.set_verification_key[0].parse().unwrap(),
                value.set_verification_key[1].parse::<u32>().unwrap().into(),
            ),
            set_zkapp_uri: value.set_zkapp_uri.parse().unwrap(),
            edit_action_state: value.edit_action_state.parse().unwrap(),
            set_token_symbol: value.set_token_symbol.parse().unwrap(),
            set_timing: value.set_timing.parse().unwrap(),
            set_voting_for: value.set_voting_for.parse().unwrap(),
            increment_nonce: value.increment_nonce.parse().unwrap(),
        }
    }
}

#[derive(GraphQLObject)]
pub struct GraphQLBalanceChange {
    pub magnitude: String,
    pub sgn: String,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLBalanceChange {
    pub magnitude: String,
    pub sgn: String,
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

impl From<MinaStateBlockchainStateValueStableV2SignedAmount> for GraphQLBalanceChange {
    fn from(value: MinaStateBlockchainStateValueStableV2SignedAmount) -> Self {
        Self {
            magnitude: value.magnitude.as_u64().to_string(),
            sgn: value.sgn.to_string(),
        }
    }
}

impl From<InputGraphQLBalanceChange> for MinaStateBlockchainStateValueStableV2SignedAmount {
    fn from(value: InputGraphQLBalanceChange) -> Self {
        Self {
            magnitude: CurrencyAmountStableV1(value.magnitude.parse::<u64>().unwrap().into()),
            sgn: value.sgn.parse().unwrap(),
        }
    }
}

impl From<MinaBaseAccountUpdatePreconditionsStableV1> for GraphQLPreconditions {
    fn from(value: MinaBaseAccountUpdatePreconditionsStableV1) -> Self {
        Self {
            network: GraphQLPreconditionsNetwork::from(value.network),
            account: GraphQLPreconditionsAccount::from(value.account),
            valid_while: if let MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Check(v) =
                value.valid_while
            {
                Some(GraphQLPreconditionsNetworkBounds {
                    upper: v.upper.as_u32() as i32,
                    lower: v.lower.as_u32() as i32,
                })
            } else {
                None
            },
        }
    }
}

impl From<InputGraphQLPreconditions> for MinaBaseAccountUpdatePreconditionsStableV1 {
    fn from(value: InputGraphQLPreconditions) -> Self {
        Self {
            network: value.network.into(),
            account: value.account.into(),
            valid_while: if let Some(v) = value.valid_while {
                MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA {
                        upper: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
                            (v.upper as u32).into(),
                        ),
                        lower: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
                            (v.lower as u32).into(),
                        ),
                    },
                )
            } else {
                MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Ignore
            },
        }
    }
}

impl From<MinaBaseAccountUpdateUpdateStableV1> for GraphQLAccountUpdateUpdate {
    fn from(value: MinaBaseAccountUpdateUpdateStableV1) -> Self {
        Self {
            app_state: value
                .app_state
                .0
                .into_iter()
                .map(|v| {
                    if let MinaBaseAccountUpdateUpdateStableV1AppStateA::Set(value) = v {
                        Some(value.to_fp().unwrap().to_decimal())
                    } else {
                        None
                    }
                })
                .collect(),
            delegate: if let MinaBaseAccountUpdateUpdateStableV1Delegate::Set(v) = value.delegate {
                Some(v.to_string())
            } else {
                None
            },
            verification_key: if let MinaBaseAccountUpdateUpdateStableV1VerificationKey::Set(v) =
                value.verification_key
            {
                Some(v.to_base64().unwrap())
            } else {
                None
            },
            permissions: if let MinaBaseAccountUpdateUpdateStableV1Permissions::Set(v) =
                value.permissions
            {
                Some(GraphQLAccountUpdateUpdatePermissions::from(*v))
            } else {
                None
            },
            zkapp_uri: if let MinaBaseAccountUpdateUpdateStableV1ZkappUri::Set(v) = value.zkapp_uri
            {
                Some(v.to_string())
            } else {
                None
            },
            token_symbol: if let MinaBaseAccountUpdateUpdateStableV1TokenSymbol::Set(v) =
                value.token_symbol
            {
                Some(v.to_string())
            } else {
                None
            },
            timing: if let MinaBaseAccountUpdateUpdateStableV1Timing::Set(v) = value.timing {
                Some(GraphQLTiming::from(*v))
            } else {
                None
            },
            voting_for: if let MinaBaseAccountUpdateUpdateStableV1VotingFor::Set(v) =
                value.voting_for
            {
                Some(v.to_string())
            } else {
                None
            },
        }
    }
}

impl From<InputGraphQLAccountUpdateUpdate> for MinaBaseAccountUpdateUpdateStableV1 {
    fn from(value: InputGraphQLAccountUpdateUpdate) -> Self {
        let app_state: Vec<_> = value
            .app_state
            .iter()
            .map(|v| {
                if let Some(v) = v {
                    MinaBaseAccountUpdateUpdateStableV1AppStateA::Set(
                        BigInt::from_decimal(&v).unwrap().into(),
                    )
                } else {
                    MinaBaseAccountUpdateUpdateStableV1AppStateA::Keep
                }
            })
            .collect();
        Self {
            app_state: PaddedSeq(app_state.try_into().unwrap()),
            delegate: if let Some(delegate) = value.delegate {
                MinaBaseAccountUpdateUpdateStableV1Delegate::Set(
                    AccountPublicKey::from_str(&delegate).unwrap().into(),
                )
            } else {
                MinaBaseAccountUpdateUpdateStableV1Delegate::Keep
            },
            verification_key: if let Some(vk) = value.verification_key {
                MinaBaseAccountUpdateUpdateStableV1VerificationKey::Set(Box::new(
                    MinaBaseVerificationKeyWireStableV1::from_base64(&vk).unwrap(),
                ))
            } else {
                MinaBaseAccountUpdateUpdateStableV1VerificationKey::Keep
            },
            permissions: if let Some(permissions) = value.permissions {
                MinaBaseAccountUpdateUpdateStableV1Permissions::Set(Box::new(
                    MinaBasePermissionsStableV2::from(permissions),
                ))
            } else {
                MinaBaseAccountUpdateUpdateStableV1Permissions::Keep
            },
            zkapp_uri: if let Some(zkapp_uri) = value.zkapp_uri {
                MinaBaseAccountUpdateUpdateStableV1ZkappUri::Set(ZkAppUri::from(zkapp_uri.as_str()))
            } else {
                MinaBaseAccountUpdateUpdateStableV1ZkappUri::Keep
            },
            token_symbol: if let Some(token_symbol) = value.token_symbol {
                MinaBaseAccountUpdateUpdateStableV1TokenSymbol::Set(TokenSymbol::from(
                    token_symbol.as_str(),
                ))
            } else {
                MinaBaseAccountUpdateUpdateStableV1TokenSymbol::Keep
            },
            timing: if let Some(timing) = value.timing {
                MinaBaseAccountUpdateUpdateStableV1Timing::Set(Box::new(
                    MinaBaseAccountUpdateUpdateTimingInfoStableV1::from(timing),
                ))
            } else {
                MinaBaseAccountUpdateUpdateStableV1Timing::Keep
            },
            voting_for: if let Some(voting_for) = value.voting_for {
                MinaBaseAccountUpdateUpdateStableV1VotingFor::Set(
                    StateHash::from_str(&voting_for).unwrap(),
                )
            } else {
                MinaBaseAccountUpdateUpdateStableV1VotingFor::Keep
            },
        }
    }
}

impl From<MinaBaseAccountUpdateFeePayerStableV1> for GraphQLFeePayer {
    fn from(value: MinaBaseAccountUpdateFeePayerStableV1) -> Self {
        Self {
            authorization: value.authorization.to_string(),
            body: GraphQLFeePayerBody {
                public_key: value.body.public_key.to_string(),
                fee: value.body.fee.as_u64().to_string(),
                valid_until: value.body.valid_until.map(|v| v.as_u32().to_string()),
                nonce: value.body.nonce.to_string(),
            },
        }
    }
}

impl From<InputGraphQLFeePayer> for MinaBaseAccountUpdateFeePayerStableV1 {
    fn from(value: InputGraphQLFeePayer) -> Self {
        Self {
            authorization: value.authorization.parse().unwrap(),
            body: MinaBaseAccountUpdateBodyFeePayerStableV1 {
                public_key: value.body.public_key.parse().unwrap(),
                fee: CurrencyFeeStableV1(value.body.fee.parse::<u64>().unwrap().into()),
                valid_until: value.body.valid_until.map(|v| {
                    MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
                        v.parse::<u32>().unwrap().into(),
                    )
                }),
                nonce: value.body.nonce.parse::<u32>().unwrap().into(),
            },
        }
    }
}

impl From<MinaBaseAccountUpdateTStableV1> for GraphQLAccountUpdate {
    fn from(value: MinaBaseAccountUpdateTStableV1) -> Self {
        Self {
            body: GraphQLAccountUpdateBody {
                public_key: value.body.public_key.to_string(),
                token_id: value.body.token_id.to_string(),
                use_full_commitment: value.body.use_full_commitment,
                increment_nonce: value.body.increment_nonce,
                update: GraphQLAccountUpdateUpdate::from(value.body.update),
                balance_change: GraphQLBalanceChange::from(value.body.balance_change),
                events: value
                    .body
                    .events
                    .0
                    .into_iter()
                    .map(|v| {
                        v.into_iter()
                            .map(|i| i.to_fp().unwrap().to_decimal())
                            .collect()
                    })
                    .collect(),
                actions: value
                    .body
                    .actions
                    .0
                    .into_iter()
                    .map(|v| {
                        v.into_iter()
                            .map(|i| i.to_fp().unwrap().to_decimal())
                            .collect()
                    })
                    .collect(),
                call_data: value.body.call_data.to_fp().unwrap().to_decimal(),
                // TODO(adonagy): figure out call depth
                call_depth: 0,
                preconditions: GraphQLPreconditions::from(value.body.preconditions),
                may_use_token: GraphQLMayUseToken::from(value.body.may_use_token),
                authorization_kind: GraphQLAuthorizationKind::from(value.body.authorization_kind),
                implicit_account_creation_fee: value.body.implicit_account_creation_fee,
            },
            authorization: GraphQLAuthorization::from(value.authorization),
        }
    }
}

impl From<InputGraphQLAccountUpdate> for MinaBaseAccountUpdateTStableV1 {
    fn from(value: InputGraphQLAccountUpdate) -> Self {
        Self {
            body: MinaBaseAccountUpdateBodyStableV1 {
                public_key: value.body.public_key.parse().unwrap(),
                token_id: value.body.token_id.parse().unwrap(),
                update: value.body.update.into(),
                balance_change: value.body.balance_change.into(),
                increment_nonce: value.body.increment_nonce,
                events: MinaBaseAccountUpdateBodyEventsStableV1(
                    value
                        .body
                        .events
                        .into_iter()
                        .map(|v| {
                            v.into_iter()
                                .map(|i| BigInt::from_decimal(&i).unwrap())
                                .collect()
                        })
                        .collect(),
                ),
                actions: MinaBaseAccountUpdateBodyEventsStableV1(
                    value
                        .body
                        .actions
                        .into_iter()
                        .map(|v| {
                            v.into_iter()
                                .map(|i| BigInt::from_decimal(&i).unwrap())
                                .collect()
                        })
                        .collect(),
                ),
                call_data: BigInt::from_decimal(&value.body.call_data).unwrap(),
                preconditions: value.body.preconditions.into(),
                use_full_commitment: value.body.use_full_commitment,
                implicit_account_creation_fee: value.body.implicit_account_creation_fee,
                may_use_token: value.body.may_use_token.into(),
                authorization_kind: value.body.authorization_kind.into(),
            },
            authorization: value.authorization.into(),
        }
    }
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
                    let account_updates = zkapp
                        .account_updates
                        .clone()
                        .into_iter()
                        .map(|v| v.elt.account_update.into())
                        .collect();
                    Some(GraphQLZkapp {
                        hash: zkapp.hash().unwrap().to_string(),
                        failure_reason,
                        id: serde_json::to_string_pretty(
                            &MinaBaseZkappCommandTStableV1WireStableV1Base64::from(zkapp.clone()),
                        )
                        .unwrap()
                        .trim_matches('"')
                        .to_string(),
                        zkapp_command: GraphQLZkappCommand {
                            memo: serde_json::to_string_pretty(&zkapp.memo)
                                .unwrap()
                                .trim_matches('"')
                                .to_string(),
                            account_updates,
                            fee_payer: GraphQLFeePayer::from(zkapp.fee_payer),
                        },
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

impl From<InputGraphQLTiming> for MinaBaseAccountUpdateUpdateTimingInfoStableV1 {
    fn from(value: InputGraphQLTiming) -> Self {
        Self {
            initial_minimum_balance: CurrencyBalanceStableV1(CurrencyAmountStableV1(
                value.initial_minimum_balance.parse::<u64>().unwrap().into(),
            )),
            cliff_time: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
                (value.cliff_time as u32).into(),
            ),
            cliff_amount: CurrencyAmountStableV1(value.cliff_amount.parse::<u64>().unwrap().into()),
            vesting_period: MinaNumbersGlobalSlotSpanStableV1::GlobalSlotSpan(
                (value.vesting_period as u32).into(),
            ),
            vesting_increment: CurrencyAmountStableV1(
                value.vesting_increment.parse::<u64>().unwrap().into(),
            ),
        }
    }
}

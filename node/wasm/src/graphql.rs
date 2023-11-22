use std::iter::Peekable;
use std::str::FromStr;

use binprot::{BinProtRead, BinProtWrite};
use gloo_utils::format::JsValueSerdeExt;
use lib::snark::utils::FpExt;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    pseq::PaddedSeq,
    string::ByteString,
    v2::{
        CurrencyAmountStableV1, CurrencyBalanceStableV1, CurrencyFeeStableV1, EpochSeed,
        LedgerHash, MinaBaseAccountBinableArgStableV2, MinaBaseAccountTimingStableV2,
        MinaBaseAccountUpdateAccountPreconditionStableV1,
        MinaBaseAccountUpdateAuthorizationKindStableV1, MinaBaseAccountUpdateBodyEventsStableV1,
        MinaBaseAccountUpdateBodyFeePayerStableV1, MinaBaseAccountUpdateBodyStableV1,
        MinaBaseAccountUpdateFeePayerStableV1, MinaBaseAccountUpdateMayUseTokenStableV1,
        MinaBaseAccountUpdatePreconditionsStableV1, MinaBaseAccountUpdateTStableV1,
        MinaBaseAccountUpdateUpdateStableV1, MinaBaseAccountUpdateUpdateStableV1AppStateA,
        MinaBaseAccountUpdateUpdateStableV1Delegate,
        MinaBaseAccountUpdateUpdateStableV1Permissions, MinaBaseAccountUpdateUpdateStableV1Timing,
        MinaBaseAccountUpdateUpdateStableV1TokenSymbol,
        MinaBaseAccountUpdateUpdateStableV1VerificationKey,
        MinaBaseAccountUpdateUpdateStableV1VotingFor, MinaBaseAccountUpdateUpdateStableV1ZkappUri,
        MinaBaseAccountUpdateUpdateTimingInfoStableV1, MinaBaseControlStableV2,
        MinaBasePendingCoinbaseHashBuilderStableV1, MinaBasePendingCoinbaseHashVersionedStableV1,
        MinaBasePermissionsAuthRequiredStableV2, MinaBasePermissionsStableV2,
        MinaBaseReceiptChainHashStableV1, MinaBaseSignedCommandMemoStableV1,
        MinaBaseUserCommandStableV2, MinaBaseVerificationKeyWireStableV1,
        MinaBaseZkappAccountStableV2, MinaBaseZkappAccountZkappUriStableV1,
        MinaBaseZkappCommandTStableV1WireStableV1,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA,
        MinaBaseZkappPreconditionAccountStableV2, MinaBaseZkappPreconditionAccountStableV2Balance,
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
        MinaBaseZkappStateValueStableV1, MinaNumbersGlobalSlotSinceGenesisMStableV1,
        MinaNumbersGlobalSlotSpanStableV1, MinaStateBlockchainStateValueStableV2SignedAmount,
        NonZeroCurvePoint, PendingCoinbaseHash, PicklesProofProofsVerifiedMaxStableV2, SgnStableV1,
        Signature, StateHash, TokenIdKeyHash, UnsignedExtendedUInt32StableV1,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_with::serde_as;
use wasm_bindgen::JsValue;

pub trait GraphqlConv {
    type GraphqlType;

    fn to_graphql(self) -> Self::GraphqlType;
    fn from_graphql(value: Self::GraphqlType) -> Self;
}

pub trait GraphqlConvJs: Sized {
    fn to_js_value(self) -> Result<JsValue, String>;

    fn from_js_value(value: JsValue) -> Result<Self, String>;
}

impl<T> GraphqlConvJs for T
where
    T: GraphqlConv,
    T::GraphqlType: Serialize,
    for<'de> T::GraphqlType: Deserialize<'de>,
{
    fn to_js_value(self) -> Result<JsValue, String> {
        JsValue::from_serde(&self.to_graphql()).map_err(|err| err.to_string())
    }

    fn from_js_value(value: JsValue) -> Result<Self, String> {
        value
            .into_serde()
            .map(Self::from_graphql)
            .map_err(|err| err.to_string())
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub public_key: NonZeroCurvePoint,
    #[deprecated(note = "please use `token_id` instead")]
    pub token: TokenIdKeyHash,
    pub token_id: TokenIdKeyHash,
    #[serde_as(as = "ZkappUri")]
    pub token_symbol: MinaBaseZkappAccountZkappUriStableV1,
    pub balance: Balance,
    pub nonce: UnsignedExtendedUInt32StableV1,
    #[serde_as(as = "ReceiptChainHash")]
    pub receipt_chain_hash: MinaBaseReceiptChainHashStableV1,
    pub delegate_account: DelegateAccount,
    pub voting_for: StateHash,
    pub timing: Timing,
    #[serde_as(as = "Permissions")]
    pub permissions: MinaBasePermissionsStableV2,
    #[serde_as(as = "Option<[BigDecimal; 8]>")]
    pub zkapp_state: Option<[BigInt; 8]>,
    #[serde_as(as = "Option<[BigDecimal; 5]>")]
    pub action_state: Option<[BigInt; 5]>,
    pub proved_state: Option<bool>,
    #[serde_as(as = "Option<ZkappUri>")]
    pub zkapp_uri: Option<MinaBaseZkappAccountZkappUriStableV1>,
    pub verification_key: Option<VerificationKey>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationKey {
    #[serde_as(as = "VerificationKeyAsBase64")]
    pub data: MinaBaseVerificationKeyWireStableV1,
    #[serde_as(as = "BigDecimal")]
    pub hash: BigInt,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub total: CurrencyBalanceStableV1,
}

#[derive(Serialize, Deserialize)]
struct SignedAmountWrapper {
    pub magnitude: CurrencyAmountStableV1,
    pub sgn: Sgn,
}

#[derive(Serialize, Deserialize)]
enum Sgn {
    Positive,
    Negative,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegateAccount {
    pub public_key: Option<NonZeroCurvePoint>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timing {
    pub initial_minimum_balance: Option<CurrencyBalanceStableV1>,
    pub cliff_time: Option<UnsignedExtendedUInt32StableV1>,
    pub cliff_amount: Option<CurrencyAmountStableV1>,
    pub vesting_period: Option<MinaNumbersGlobalSlotSpanStableV1>,
    pub vesting_increment: Option<CurrencyAmountStableV1>,
}

impl From<MinaBaseAccountTimingStableV2> for Timing {
    fn from(v: MinaBaseAccountTimingStableV2) -> Self {
        match v {
            MinaBaseAccountTimingStableV2::Untimed => Self {
                initial_minimum_balance: None,
                cliff_time: None,
                cliff_amount: None,
                vesting_period: None,
                vesting_increment: None,
            },
            MinaBaseAccountTimingStableV2::Timed {
                initial_minimum_balance,
                cliff_time: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(cliff_time),
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => Self {
                initial_minimum_balance: Some(initial_minimum_balance),
                cliff_time: Some(cliff_time),
                cliff_amount: Some(cliff_amount),
                vesting_period: Some(vesting_period),
                vesting_increment: Some(vesting_increment),
            },
        }
    }
}

impl From<Timing> for MinaBaseAccountTimingStableV2 {
    fn from(v: Timing) -> Self {
        match (
            v.initial_minimum_balance,
            v.cliff_time,
            v.cliff_amount,
            v.vesting_period,
            v.vesting_increment,
        ) {
            (
                Some(initial_minimum_balance),
                Some(cliff_time),
                Some(cliff_amount),
                Some(vesting_period),
                Some(vesting_increment),
            ) => Self::Timed {
                initial_minimum_balance,
                cliff_time: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(cliff_time),
                cliff_amount,
                vesting_period,
                vesting_increment,
            },
            _ => Self::Untimed,
        }
    }
}

impl Timing {
    pub fn into_account_update(self) -> Option<MinaBaseAccountUpdateUpdateTimingInfoStableV1> {
        match (
            self.initial_minimum_balance,
            self.cliff_time,
            self.cliff_amount,
            self.vesting_period,
            self.vesting_increment,
        ) {
            (
                Some(initial_minimum_balance),
                Some(cliff_time),
                Some(cliff_amount),
                Some(vesting_period),
                Some(vesting_increment),
            ) => Some(MinaBaseAccountUpdateUpdateTimingInfoStableV1 {
                initial_minimum_balance,
                cliff_time: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(cliff_time),
                cliff_amount,
                vesting_period,
                vesting_increment,
            }),
            _ => None,
        }
    }
}

impl GraphqlConv for MinaBaseAccountBinableArgStableV2 {
    type GraphqlType = Account;

    fn to_graphql(self) -> Self::GraphqlType {
        let (zkapp_state, action_state, proved_state, zkapp_uri, verification_key) =
            self.zkapp.map_or((None, None, None, None, None), |v| {
                let key = v.verification_key.map(|v| VerificationKey {
                    hash: ledger::VerificationKey::from(&v).hash().into(),
                    data: v,
                });
                (
                    Some(v.app_state),
                    Some(v.action_state),
                    Some(v.proved_state),
                    Some(v.zkapp_uri),
                    key,
                )
            });
        Account {
            public_key: self.public_key,
            token: self.token_id.clone(),
            token_id: self.token_id,
            token_symbol: self.token_symbol,
            balance: Balance {
                total: self.balance,
            },
            nonce: self.nonce,
            receipt_chain_hash: self.receipt_chain_hash,
            delegate_account: DelegateAccount {
                public_key: self.delegate,
            },
            voting_for: self.voting_for,
            timing: self.timing.into(),
            permissions: self.permissions,
            zkapp_state: zkapp_state.map(|v| v.0 .0),
            action_state: action_state.map(|v| v.0),
            proved_state,
            zkapp_uri,
            verification_key,
        }
    }

    fn from_graphql(_value: Self::GraphqlType) -> Self {
        todo!()
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zkapp {
    pub account_updates: Vec<ZkappAccountUpdate>,
    pub fee_payer: ZkappFeePayer,
    #[serde_as(as = "UserCommandMemo")]
    pub memo: MinaBaseSignedCommandMemoStableV1,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappAccountUpdate {
    pub body: ZkappAccountUpdateBody,
    pub authorization: ZkappAccountUpdateAuthorization,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappAccountUpdateAuthorization {
    pub signature: Option<Signature>,
    #[serde_as(as = "Option<ProofAsBase64Sexp>")]
    pub proof: Option<PicklesProofProofsVerifiedMaxStableV2>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappAccountUpdateBody {
    pub public_key: NonZeroCurvePoint,
    pub token_id: TokenIdKeyHash,
    pub update: ZkappAccountUpdateUpdate,
    #[serde_as(as = "SignedAmount")]
    pub balance_change: MinaStateBlockchainStateValueStableV2SignedAmount,
    pub increment_nonce: bool,
    #[serde_as(as = "Vec<Vec<BigDecimal>>")]
    pub events: Vec<Vec<BigInt>>,
    #[serde_as(as = "Vec<Vec<BigDecimal>>")]
    pub actions: Vec<Vec<BigInt>>,
    #[serde_as(as = "BigDecimal")]
    pub call_data: BigInt,
    pub call_depth: u16,
    pub preconditions: ZkappPreconditions,
    pub use_full_commitment: bool,
    pub implicit_account_creation_fee: bool,
    pub may_use_token: MayUseToken,
    pub authorization_kind: AuthorizationKind,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MayUseToken {
    pub inherit_from_parent: bool,
    pub parents_own_token: bool,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationKind {
    pub is_proved: bool,
    pub is_signed: bool,
    #[serde_as(as = "Option<BigDecimal>")]
    pub verification_key_hash: Option<BigInt>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappAccountUpdateUpdate {
    #[serde_as(as = "[Option<BigDecimal>; 8]")]
    pub app_state: [Option<BigInt>; 8],
    pub delegate: Option<NonZeroCurvePoint>,
    pub verification_key: Option<VerificationKey>,
    #[serde_as(as = "Option<Permissions>")]
    pub permissions: Option<MinaBasePermissionsStableV2>,
    #[serde_as(as = "Option<ZkappUri>")]
    pub zkapp_uri: Option<MinaBaseZkappAccountZkappUriStableV1>,
    #[serde_as(as = "Option<ZkappUri>")]
    pub token_symbol: Option<MinaBaseZkappAccountZkappUriStableV1>,
    pub timing: Option<Timing>,
    pub voting_for: Option<StateHash>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappPreconditions {
    pub network: ZkappPreconditionsNetwork,
    pub account: ZkappPreconditionsAccount,
    pub valid_while: Option<PreconditionClosedInterval<UnsignedExtendedUInt32StableV1>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappPreconditionsNetwork {
    pub snarked_ledger_hash: Option<LedgerHash>,
    pub blockchain_length: Option<PreconditionClosedInterval<UnsignedExtendedUInt32StableV1>>,
    pub min_window_density: Option<PreconditionClosedInterval<UnsignedExtendedUInt32StableV1>>,
    pub total_currency: Option<PreconditionClosedInterval<CurrencyAmountStableV1>>,
    pub global_slot_since_genesis:
        Option<PreconditionClosedInterval<UnsignedExtendedUInt32StableV1>>,
    pub staking_epoch_data: ZkappPreconditionsNetworkEpoch,
    pub next_epoch_data: ZkappPreconditionsNetworkEpoch,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappPreconditionsNetworkEpoch {
    pub ledger: ZkappPreconditionsNetworkEpochLedger,
    pub seed: Option<EpochSeed>,
    pub start_checkpoint: Option<StateHash>,
    pub lock_checkpoint: Option<StateHash>,
    pub epoch_length: Option<PreconditionClosedInterval<UnsignedExtendedUInt32StableV1>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappPreconditionsNetworkEpochLedger {
    pub hash: Option<LedgerHash>,
    pub total_currency: Option<PreconditionClosedInterval<CurrencyAmountStableV1>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreconditionClosedInterval<T> {
    pub lower: T,
    pub upper: T,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappPreconditionsAccount {
    pub balance: Option<PreconditionClosedInterval<CurrencyBalanceStableV1>>,
    pub nonce: Option<PreconditionClosedInterval<UnsignedExtendedUInt32StableV1>>,
    #[serde_as(as = "Option<ReceiptChainHash>")]
    pub receipt_chain_hash: Option<MinaBaseReceiptChainHashStableV1>,
    pub delegate: Option<NonZeroCurvePoint>,
    #[serde_as(as = "[Option<BigDecimal>; 8]")]
    pub state: [Option<BigInt>; 8],
    #[serde_as(as = "Option<BigDecimal>")]
    pub action_state: Option<BigInt>,
    pub proved_state: Option<bool>,
    pub is_new: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappFeePayer {
    pub body: ZkappFeePayerBody,
    pub authorization: Signature,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkappFeePayerBody {
    pub public_key: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub valid_until: Option<UnsignedExtendedUInt32StableV1>,
    pub nonce: UnsignedExtendedUInt32StableV1,
}

impl From<PreconditionClosedInterval<UnsignedExtendedUInt32StableV1>>
    for MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA
{
    fn from(value: PreconditionClosedInterval<UnsignedExtendedUInt32StableV1>) -> Self {
        Self {
            lower: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(value.lower),
            upper: MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(value.upper),
        }
    }
}
impl From<ZkappPreconditionsAccount> for MinaBaseAccountUpdateAccountPreconditionStableV1 {
    fn from(v: ZkappPreconditionsAccount) -> Self {
        Self::Full(Box::new(MinaBaseZkappPreconditionAccountStableV2 {
            balance: match v.balance {
                None => MinaBaseZkappPreconditionAccountStableV2Balance::Ignore,
                Some(v) => MinaBaseZkappPreconditionAccountStableV2Balance::Check(
                    MinaBaseZkappPreconditionAccountStableV2BalanceA {
                        lower: v.lower,
                        upper: v.upper,
                    },
                ),
            },
            nonce: match v.nonce {
                None => MinaBaseZkappPreconditionProtocolStateStableV1Length::Ignore,
                Some(v) => MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                        lower: v.lower,
                        upper: v.upper,
                    },
                ),
            },
            receipt_chain_hash: match v.receipt_chain_hash {
                None => MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash::Ignore,
                Some(v) => MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash::Check(v),
            },
            delegate: match v.delegate {
                None => MinaBaseZkappPreconditionAccountStableV2Delegate::Ignore,
                Some(v) => MinaBaseZkappPreconditionAccountStableV2Delegate::Check(v),
            },
            state: PaddedSeq(v.state.map(|v| match v {
                None => MinaBaseZkappPreconditionAccountStableV2StateA::Ignore,
                Some(v) => MinaBaseZkappPreconditionAccountStableV2StateA::Check(v),
            })),
            action_state: match v.action_state {
                None => MinaBaseZkappPreconditionAccountStableV2StateA::Ignore,
                Some(v) => MinaBaseZkappPreconditionAccountStableV2StateA::Check(v),
            },
            proved_state: match v.proved_state {
                None => MinaBaseZkappPreconditionAccountStableV2ProvedState::Ignore,
                Some(v) => MinaBaseZkappPreconditionAccountStableV2ProvedState::Check(v),
            },
            is_new: match v.is_new {
                None => MinaBaseZkappPreconditionAccountStableV2ProvedState::Ignore,
                Some(v) => MinaBaseZkappPreconditionAccountStableV2ProvedState::Check(v),
            },
        }))
    }
}

impl From<ZkappPreconditionsNetworkEpoch>
    for MinaBaseZkappPreconditionProtocolStateEpochDataStableV1
{
    fn from(v: ZkappPreconditionsNetworkEpoch) -> Self {
        Self {
            ledger: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger {
                hash: match v.ledger.hash {
                    None => MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Ignore,
                    Some(v) => {
                        MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Check(v)
                    }
                },
                total_currency: match v.ledger.total_currency {
                    None => MinaBaseZkappPreconditionProtocolStateStableV1Amount::Ignore,
                    Some(v) => MinaBaseZkappPreconditionProtocolStateStableV1Amount::Check(
                        MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
                            lower: v.lower,
                            upper: v.upper,
                        },
                    ),
                },
            },
            seed: match v.seed {
                None => MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed::Ignore,
                Some(v) => {
                    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed::Check(v)
                }
            },
            start_checkpoint: match v.start_checkpoint {
                None => {
                    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Ignore
                }
                Some(v) => {
                    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Check(v)
                }
            },
            lock_checkpoint: match v.lock_checkpoint {
                None => {
                    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Ignore
                }
                Some(v) => {
                    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint::Check(v)
                }
            },
            epoch_length: match v.epoch_length {
                None => MinaBaseZkappPreconditionProtocolStateStableV1Length::Ignore,
                Some(v) => MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                    MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                        lower: v.lower,
                        upper: v.upper,
                    },
                ),
            },
        }
    }
}

impl From<ZkappPreconditions> for MinaBaseAccountUpdatePreconditionsStableV1 {
    fn from(v: ZkappPreconditions) -> Self {
        Self {
            network: MinaBaseZkappPreconditionProtocolStateStableV1 {
                snarked_ledger_hash: match v.network.snarked_ledger_hash {
                    None => MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Ignore,
                    Some(v) => {
                        MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash::Check(v)
                    }
                },
                blockchain_length: match v.network.blockchain_length {
                    None => MinaBaseZkappPreconditionProtocolStateStableV1Length::Ignore,
                    Some(v) => MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                        MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                            lower: v.lower,
                            upper: v.upper,
                        },
                    ),
                },
                min_window_density: match v.network.min_window_density {
                    None => MinaBaseZkappPreconditionProtocolStateStableV1Length::Ignore,
                    Some(v) => MinaBaseZkappPreconditionProtocolStateStableV1Length::Check(
                        MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                            lower: v.lower,
                            upper: v.upper,
                        },
                    ),
                },
                total_currency: match v.network.total_currency {
                    None => MinaBaseZkappPreconditionProtocolStateStableV1Amount::Ignore,
                    Some(v) => MinaBaseZkappPreconditionProtocolStateStableV1Amount::Check(
                        MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
                            lower: v.lower,
                            upper: v.upper,
                        },
                    ),
                },
                global_slot_since_genesis: match v.network.global_slot_since_genesis {
                    None => MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Ignore,
                    Some(v) => {
                        MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Check(v.into())
                    }
                },
                staking_epoch_data: v.network.staking_epoch_data.into(),
                next_epoch_data: v.network.next_epoch_data.into(),
            },
            account: v.account.into(),
            valid_while: match v.valid_while {
                None => MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Ignore,
                Some(v) => {
                    MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot::Check(v.into())
                }
            },
        }
    }
}

impl From<AuthorizationKind> for MinaBaseAccountUpdateAuthorizationKindStableV1 {
    fn from(v: AuthorizationKind) -> Self {
        if v.is_signed {
            Self::Signature
        } else if v.is_proved {
            if let Some(vk) = v.verification_key_hash {
                Self::Proof(vk)
            } else {
                Self::NoneGiven
            }
        } else {
            Self::NoneGiven
        }
    }
}

impl From<MayUseToken> for MinaBaseAccountUpdateMayUseTokenStableV1 {
    fn from(v: MayUseToken) -> Self {
        if v.inherit_from_parent {
            Self::InheritFromParent
        } else if v.parents_own_token {
            Self::ParentsOwnToken
        } else {
            Self::No
        }
    }
}

impl From<ZkappAccountUpdateUpdate> for MinaBaseAccountUpdateUpdateStableV1 {
    fn from(update: ZkappAccountUpdateUpdate) -> Self {
        Self {
            app_state: PaddedSeq(update.app_state.map(|v| match v {
                None => MinaBaseAccountUpdateUpdateStableV1AppStateA::Keep,
                Some(v) => MinaBaseAccountUpdateUpdateStableV1AppStateA::Set(v),
            })),
            delegate: match update.delegate {
                None => MinaBaseAccountUpdateUpdateStableV1Delegate::Keep,
                Some(v) => MinaBaseAccountUpdateUpdateStableV1Delegate::Set(v),
            },
            verification_key: match update.verification_key {
                None => MinaBaseAccountUpdateUpdateStableV1VerificationKey::Keep,
                Some(v) => MinaBaseAccountUpdateUpdateStableV1VerificationKey::Set(v.data.into()),
            },
            permissions: match update.permissions {
                None => MinaBaseAccountUpdateUpdateStableV1Permissions::Keep,
                Some(v) => MinaBaseAccountUpdateUpdateStableV1Permissions::Set(v.into()),
            },
            zkapp_uri: match update.zkapp_uri {
                None => MinaBaseAccountUpdateUpdateStableV1ZkappUri::Keep,
                Some(v) => MinaBaseAccountUpdateUpdateStableV1ZkappUri::Set(v.0),
            },
            token_symbol: match update.token_symbol {
                None => MinaBaseAccountUpdateUpdateStableV1TokenSymbol::Keep,
                Some(v) => MinaBaseAccountUpdateUpdateStableV1TokenSymbol::Set(v),
            },
            timing: match update.timing {
                None => MinaBaseAccountUpdateUpdateStableV1Timing::Keep,
                Some(v) => MinaBaseAccountUpdateUpdateStableV1Timing::Set(
                    v.into_account_update().unwrap().into(),
                ),
            },
            voting_for: match update.voting_for {
                None => MinaBaseAccountUpdateUpdateStableV1VotingFor::Keep,
                Some(v) => MinaBaseAccountUpdateUpdateStableV1VotingFor::Set(v),
            },
        }
    }
}

impl From<ZkappAccountUpdateBody> for MinaBaseAccountUpdateBodyStableV1 {
    fn from(v: ZkappAccountUpdateBody) -> Self {
        Self {
            public_key: v.public_key,
            token_id: v.token_id,
            update: v.update.into(),
            balance_change: v.balance_change,
            increment_nonce: v.increment_nonce,
            use_full_commitment: v.use_full_commitment,
            implicit_account_creation_fee: v.implicit_account_creation_fee,
            events: MinaBaseAccountUpdateBodyEventsStableV1(v.events),
            actions: MinaBaseAccountUpdateBodyEventsStableV1(v.actions),
            call_data: v.call_data,
            preconditions: v.preconditions.into(),
            may_use_token: v.may_use_token.into(),
            authorization_kind: v.authorization_kind.into(),
        }
    }
}

impl From<ZkappAccountUpdateAuthorization> for MinaBaseControlStableV2 {
    fn from(auth: ZkappAccountUpdateAuthorization) -> Self {
        match (auth.signature, auth.proof) {
            (Some(sig), _) => MinaBaseControlStableV2::Signature(sig),
            (_, Some(proof)) => MinaBaseControlStableV2::Proof(Box::new(proof)),
            _ => MinaBaseControlStableV2::NoneGiven,
        }
    }
}

fn updates_list_to_map(
    updates: &mut Peekable<impl Iterator<Item = ZkappAccountUpdate>>,
) -> Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA> {
    let mut res = vec![];
    while let Some(update) = updates.next() {
        let mut calls = vec![];
        while updates
            .peek()
            .map_or(false, |next| next.body.call_depth > update.body.call_depth)
        {
            calls.extend(updates_list_to_map(updates));
        }
        res.push(
            MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA {
                elt: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
                    account_update: MinaBaseAccountUpdateTStableV1 {
                        body: update.body.into(),
                        authorization: update.authorization.into(),
                    },
                    account_update_digest: (),
                    calls,
                }
                .into(),
                stack_hash: (),
            },
        );
    }
    res
}

impl GraphqlConv for MinaBaseZkappCommandTStableV1WireStableV1 {
    type GraphqlType = Zkapp;

    fn to_graphql(self) -> Self::GraphqlType {
        todo!()
    }

    fn from_graphql(value: Self::GraphqlType) -> Self {
        Self {
            account_updates: updates_list_to_map(&mut value.account_updates.into_iter().peekable())
                .into_iter()
                .map(
                    |v| MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA {
                        elt: *v.elt,
                        stack_hash: v.stack_hash,
                    },
                )
                .collect(),
            fee_payer: MinaBaseAccountUpdateFeePayerStableV1 {
                body: MinaBaseAccountUpdateBodyFeePayerStableV1 {
                    public_key: value.fee_payer.body.public_key,
                    fee: value.fee_payer.body.fee,
                    valid_until: value
                        .fee_payer
                        .body
                        .valid_until
                        .map(MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis),
                    nonce: value.fee_payer.body.nonce,
                },
                authorization: value.fee_payer.authorization,
            },
            memo: value.memo,
        }
    }
}

serde_with::serde_conv!(
    BigDecimal,
    BigInt,
    |v: &BigInt| v.to_fp().unwrap().to_decimal(),
    |s: String| -> Result<_, String> { Fp::from_decimal(&s).map(|v| v.into()) }
);

serde_with::serde_conv!(
    ReceiptChainHash,
    MinaBaseReceiptChainHashStableV1,
    |v: &MinaBaseReceiptChainHashStableV1| PendingCoinbaseHash::from(
        MinaBasePendingCoinbaseHashVersionedStableV1(MinaBasePendingCoinbaseHashBuilderStableV1(
            v.0.clone()
        ))
    ),
    |s: PendingCoinbaseHash| -> Result<_, std::convert::Infallible> {
        Ok(MinaBaseReceiptChainHashStableV1(s.into_inner().0 .0))
    }
);

serde_with::serde_conv!(
    ZkappUri,
    MinaBaseZkappAccountZkappUriStableV1,
    |v: &MinaBaseZkappAccountZkappUriStableV1| String::try_from(&v.0)
        .unwrap_or_else(|_| "<not utf8>".to_owned()),
    |s: String| -> Result<_, std::convert::Infallible> {
        Ok(MinaBaseZkappAccountZkappUriStableV1(s.into_bytes().into()))
    }
);

serde_with::serde_conv!(
    UserCommandMemo,
    MinaBaseSignedCommandMemoStableV1,
    |v: &MinaBaseSignedCommandMemoStableV1| {
        bs58::encode(&v.0).with_check_version(0x14).into_string()
    },
    |s: String| -> Result<_, bs58::decode::Error> {
        let bytes = bs58::decode(s).with_check(Some(0x14)).into_vec()?[1..].to_vec();
        Ok(MinaBaseSignedCommandMemoStableV1(bytes.into()))
    }
);

serde_with::serde_conv!(
    pub TransactionAsBase64,
    MinaBaseUserCommandStableV2,
    |v: &MinaBaseUserCommandStableV2| {
        use base64::{engine::general_purpose, Engine as _};
        let mut bin = vec![];
        v.binprot_write(&mut bin).unwrap();
        general_purpose::STANDARD.encode(bin)
    },
    |s: String| -> Result<_, String> {
        use base64::{engine::general_purpose, Engine as _};
        let bin = general_purpose::STANDARD.decode(s).map_err(|err| err.to_string())?;
        MinaBaseUserCommandStableV2::binprot_read(&mut bin.as_slice()).map_err(|err| err.to_string())
    }
);

serde_with::serde_conv!(
    VerificationKeyAsBase64,
    MinaBaseVerificationKeyWireStableV1,
    |v: &MinaBaseVerificationKeyWireStableV1| {
        use base64::{engine::general_purpose, Engine as _};
        let mut bin = vec![];
        v.binprot_write(&mut bin).unwrap();
        general_purpose::STANDARD.encode(bin)
    },
    |s: String| -> Result<_, String> {
        use base64::{engine::general_purpose, Engine as _};
        let bin = general_purpose::STANDARD
            .decode(s)
            .map_err(|err| err.to_string())?;
        MinaBaseVerificationKeyWireStableV1::binprot_read(&mut bin.as_slice())
            .map_err(|err| err.to_string())
    }
);

serde_with::serde_conv!(
    ProofAsBase64Sexp,
    PicklesProofProofsVerifiedMaxStableV2,
    |v: &PicklesProofProofsVerifiedMaxStableV2| { todo!() },
    |s: String| -> Result<_, String> {
        // TODO: avoid conversion and instead impl serializer for `rsexp::Sexp`.
        fn conv(v: rsexp::Sexp) -> Result<serde_sexpr::Value, std::string::FromUtf8Error> {
            Ok(match v {
                rsexp::Sexp::Atom(v) => serde_sexpr::Value::Sym(String::from_utf8(v)?),
                rsexp::Sexp::List(list) => serde_sexpr::Value::List(
                    list.into_iter().map(conv).collect::<Result<Vec<_>, _>>()?,
                ),
            })
        }

        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD
            .decode(s)
            .map_err(|err| err.to_string())
            .and_then(|b| rsexp::from_slice(&b).map_err(|err| format!("{err:?}")))
            .and_then(|sexp| conv(sexp).map_err(|err| err.to_string()))
            .and_then(|sexp| serde_sexpr::from_value(sexp).map_err(|err| err.to_string()))
    }
);

serde_with::serde_conv!(
    SignedAmount,
    MinaStateBlockchainStateValueStableV2SignedAmount,
    |v: &MinaStateBlockchainStateValueStableV2SignedAmount| {
        SignedAmountWrapper {
            magnitude: v.magnitude.clone(),
            sgn: match v.sgn {
                SgnStableV1::Pos => Sgn::Positive,
                SgnStableV1::Neg => Sgn::Negative,
            },
        }
    },
    |v: SignedAmountWrapper| -> Result<_, std::convert::Infallible> {
        Ok(MinaStateBlockchainStateValueStableV2SignedAmount {
            magnitude: v.magnitude,
            sgn: match v.sgn {
                Sgn::Positive => SgnStableV1::Pos,
                Sgn::Negative => SgnStableV1::Neg,
            },
        })
    }
);

serde_with::serde_conv!(
    Permissions,
    MinaBasePermissionsStableV2,
    |v: &MinaBasePermissionsStableV2| { json_keys_to_camel_case(serde_json::to_value(v).unwrap()) },
    |v: JsonValue| -> Result<_, serde_json::Error> {
        serde_json::from_value(json_keys_to_snake_case(v))
    }
);

fn json_key_map_rec(value: JsonValue, f: fn(String) -> String) -> JsonValue {
    match value {
        JsonValue::Array(v) => {
            let v = v.into_iter().map(|v| json_key_map_rec(v, f)).collect();
            JsonValue::Array(v)
        }
        JsonValue::Object(v) => {
            let v = v
                .into_iter()
                .map(|(k, v)| (f(k), json_key_map_rec(v, f)))
                .collect();
            JsonValue::Object(v)
        }
        v => v,
    }
}

fn json_keys_to_snake_case(value: JsonValue) -> JsonValue {
    use convert_case::{Case, Casing};
    json_key_map_rec(value, |k| k.to_case(Case::Snake))
}

fn json_keys_to_camel_case(value: JsonValue) -> JsonValue {
    use convert_case::{Case, Casing};
    json_key_map_rec(value, |k| k.to_case(Case::Camel))
}

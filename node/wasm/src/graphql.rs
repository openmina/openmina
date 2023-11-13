use std::str::FromStr;

use binprot::BinProtWrite;
use gloo_utils::format::JsValueSerdeExt;
use lib::snark::utils::FpExt;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    pseq::PaddedSeq,
    string::ByteString,
    v2::{
        CurrencyAmountStableV1, CurrencyBalanceStableV1, MinaBaseAccountBinableArgStableV2,
        MinaBaseAccountTimingStableV2, MinaBasePendingCoinbaseHashBuilderStableV1,
        MinaBasePendingCoinbaseHashVersionedStableV1, MinaBasePermissionsStableV2,
        MinaBaseReceiptChainHashStableV1, MinaBaseZkappAccountStableV2,
        MinaBaseZkappAccountZkappUriStableV1, MinaBaseZkappStateValueStableV1,
        MinaNumbersGlobalSlotSinceGenesisMStableV1, MinaNumbersGlobalSlotSpanStableV1,
        NonZeroCurvePoint, PendingCoinbaseHash, StateHash, TokenIdKeyHash,
        UnsignedExtendedUInt32StableV1,
    },
};
use serde::{Deserialize, Serialize};
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
    pub verification_key: String,
    #[serde_as(as = "BigDecimal")]
    pub hash: BigInt,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub total: CurrencyBalanceStableV1,
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
    pub cliff_time: Option<MinaNumbersGlobalSlotSinceGenesisMStableV1>,
    pub cliff_amount: Option<CurrencyAmountStableV1>,
    pub vesting_period: Option<MinaNumbersGlobalSlotSpanStableV1>,
    pub vesting_increment: Option<CurrencyAmountStableV1>,
}

impl Timing {
    pub fn new(v: MinaBaseAccountTimingStableV2) -> Self {
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
                cliff_time,
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

impl GraphqlConv for MinaBaseAccountBinableArgStableV2 {
    type GraphqlType = Account;

    fn to_graphql(self) -> Self::GraphqlType {
        let (zkapp_state, action_state, proved_state, zkapp_uri, verification_key) =
            self.zkapp.map_or((None, None, None, None, None), |v| {
                let key = v.verification_key.map(|v| VerificationKey {
                    verification_key: {
                        use base64::{engine::general_purpose, Engine as _};
                        let mut bin = vec![];
                        v.binprot_write(&mut bin).unwrap();
                        general_purpose::STANDARD.encode(bin)
                    },
                    hash: ledger::VerificationKey::from(&v).hash().into(),
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
            timing: Timing::new(self.timing),
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
        Ok(MinaBaseReceiptChainHashStableV1(s.0 .0.clone()))
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

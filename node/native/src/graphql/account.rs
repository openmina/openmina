use juniper::{GraphQLInputObject, GraphQLObject};
use ledger::FpExt;
use mina_p2p_messages::v2::{
    MinaBaseAccountUpdateUpdateTimingInfoStableV1, MinaBaseVerificationKeyWireStableV1Base64, ReceiptChainHash, TokenIdKeyHash
};

#[derive(GraphQLObject)]
#[graphql(description = "A Mina account")]
pub struct GraphQLAccount {
    pub public_key: String,
    pub token_id: String,
    pub token: String,
    pub token_symbol: String,
    pub balance: GraphQLBalance,
    pub nonce: String,
    pub receipt_chain_hash: String,
    // TODO(adonagy): this should be GraphQLAccount recursively
    pub delegate_account: Option<GraphQLDelegateAccount>,
    pub voting_for: String,
    pub timing: GraphQLTiming,
    pub permissions: GraphQLPermissions,
    // can we flatten?
    // pub zkapp: Option<GraphQLZkAppAccount>,
    pub zkapp_state: Option<Vec<String>>,
    pub verification_key: Option<GraphQLVerificationKey>,
    pub action_state: Option<Vec<String>>,
    pub proved_state: Option<bool>,
    pub zkapp_uri: Option<String>,
}

#[derive(GraphQLObject)]
pub struct GraphQLDelegateAccount {
    pub public_key: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLTiming {
    // pub is_timed: bool,
    pub initial_minimum_balance: Option<String>,
    pub cliff_time: Option<i32>,
    pub cliff_amount: Option<String>,
    pub vesting_period: Option<i32>,
    pub vesting_increment: Option<String>,
}

#[derive(GraphQLInputObject)]
pub struct InputGraphQLTiming {
    // pub is_timed: bool,
    pub initial_minimum_balance: Option<String>,
    pub cliff_time: Option<i32>,
    pub cliff_amount: Option<String>,
    pub vesting_period: Option<i32>,
    pub vesting_increment: Option<String>,
}

impl From<MinaBaseAccountUpdateUpdateTimingInfoStableV1> for GraphQLTiming {
    fn from(value: MinaBaseAccountUpdateUpdateTimingInfoStableV1) -> Self {
        Self {
            initial_minimum_balance: Some(value.initial_minimum_balance.0.as_u64().to_string()),
            cliff_time: Some(value.cliff_time.as_u32() as i32),
            cliff_amount: Some(value.cliff_amount.as_u64().to_string()),
            vesting_period: Some(value.vesting_period.as_u32() as i32),
            vesting_increment: Some(value.vesting_increment.0.as_u64().to_string()),
        }
    }
}
#[derive(GraphQLObject)]
pub struct GraphQLPermissions {
    pub edit_state: String,
    pub access: String,
    pub send: String,
    pub receive: String,
    pub set_delegate: String,
    pub set_permissions: String,
    pub set_verification_key: GraphQLSetVerificationKey,
    pub set_zkapp_uri: String,
    pub edit_action_state: String,
    pub set_token_symbol: String,
    pub increment_nonce: String,
    pub set_voting_for: String,
    pub set_timing: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLSetVerificationKey {
    pub auth: String,
    pub txn_version: String,
}

#[derive(GraphQLObject)]
pub struct GraphQLBalance {
    pub total: String,
}

// #[derive(GraphQLObject)]
// pub struct GraphQLZkAppAccount {
//     pub app_state: Vec<String>,
//     pub verification_key: Option<GraphQLVerificationKey>,
//     pub zkapp_version: i32,
//     pub action_state: Vec<String>,
//     pub last_action_slot: i32,
//     pub proved_state: bool,
//     pub zkapp_uri: String,
// }

#[derive(GraphQLObject)]
pub struct GraphQLVerificationKey {
    // pub max_proofs_verified: String,
    // pub actual_wrap_domain_size: String,
    // pub wrap_index: String,
    pub verification_key: String,
    pub hash: String,
}

impl From<ledger::SetVerificationKey<ledger::AuthRequired>> for GraphQLSetVerificationKey {
    fn from(value: ledger::SetVerificationKey<ledger::AuthRequired>) -> Self {
        Self {
            auth: value.auth.to_string(),
            txn_version: value.txn_version.as_u32().to_string(),
        }
    }
}

impl From<ledger::Permissions<ledger::AuthRequired>> for GraphQLPermissions {
    fn from(value: ledger::Permissions<ledger::AuthRequired>) -> Self {
        Self {
            edit_state: value.edit_state.to_string(),
            access: value.access.to_string(),
            send: value.send.to_string(),
            receive: value.receive.to_string(),
            set_delegate: value.set_delegate.to_string(),
            set_permissions: value.set_permissions.to_string(),
            set_verification_key: GraphQLSetVerificationKey::from(value.set_verification_key),
            set_zkapp_uri: value.set_zkapp_uri.to_string(),
            edit_action_state: value.edit_action_state.to_string(),
            set_token_symbol: value.set_token_symbol.to_string(),
            increment_nonce: value.increment_nonce.to_string(),
            set_voting_for: value.set_voting_for.to_string(),
            set_timing: value.set_timing.to_string(),
        }
    }
}

impl From<ledger::Timing> for GraphQLTiming {
    fn from(value: ledger::Timing) -> Self {
        match value {
            ledger::Timing::Untimed => Self {
                initial_minimum_balance: None,
                vesting_period: None,
                cliff_time: None,
                cliff_amount: None,
                vesting_increment: None,
            },
            ledger::Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => Self {
                initial_minimum_balance: Some(initial_minimum_balance.as_u64().to_string()),
                cliff_time: Some(cliff_time.as_u32() as i32),
                cliff_amount: Some(cliff_amount.as_u64().to_string()),
                vesting_period: Some(vesting_period.as_u32() as i32),
                vesting_increment: Some(vesting_increment.as_u64().to_string()),
            },
        }
    }
}

// TODO(adonagy)
impl From<ledger::scan_state::currency::Balance> for GraphQLBalance {
    fn from(value: ledger::scan_state::currency::Balance) -> Self {
        Self {
            total: value.as_u64().to_string(),
        }
    }
}

impl From<ledger::Account> for GraphQLAccount {
    fn from(value: ledger::Account) -> Self {
        Self {
            public_key: value.public_key.into_address(),
            token_id: TokenIdKeyHash::from(value.token_id.clone()).to_string(),
            token: TokenIdKeyHash::from(value.token_id).to_string(),
            token_symbol: value.token_symbol.0,
            balance: GraphQLBalance::from(value.balance),
            nonce: value.nonce.as_u32().to_string(),
            receipt_chain_hash: ReceiptChainHash::from(value.receipt_chain_hash).to_string(),
            delegate_account: value.delegate.map(|d| GraphQLDelegateAccount {
                public_key: d.into_address(),
            }),
            voting_for: value.voting_for.to_base58check(),
            timing: GraphQLTiming::from(value.timing),
            permissions: GraphQLPermissions::from(value.permissions),
            // zkapp: value.zkapp.map(GraphQLZkAppAccount::from),
            // TODO: keep as array?
            zkapp_state: value.zkapp.clone().map(|zkapp| {
                zkapp
                    .app_state
                    .into_iter()
                    .map(|v| v.to_decimal())
                    .collect::<Vec<_>>()
            }),
            verification_key: value.zkapp.clone().and_then(|zkapp| {
                zkapp.verification_key.map(|vk| {
                    let ser = serde_json::to_string_pretty(
                        &MinaBaseVerificationKeyWireStableV1Base64::from(vk.clone()),
                    )
                    .unwrap()
                    .trim_matches('"')
                    .to_string();
                    GraphQLVerificationKey {
                        verification_key: ser,
                        hash: vk.digest().to_decimal(),
                    }
                })
            }),
            action_state: value.zkapp.clone().map(|zkapp| {
                zkapp
                    .action_state
                    .into_iter()
                    .map(|v| v.to_decimal())
                    .collect::<Vec<_>>()
            }),
            proved_state: value.zkapp.clone().map(|zkapp| zkapp.proved_state),
            zkapp_uri: value.zkapp.map(|zkapp| zkapp.zkapp_uri.to_string()),
        }
    }
}

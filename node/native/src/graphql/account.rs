use juniper::GraphQLObject;
use mina_p2p_messages::v2::{ReceiptChainHash, TokenIdKeyHash};

#[derive(GraphQLObject)]
#[graphql(description = "A Mina account")]
pub struct GraphQLAccount {
    pub public_key: String,
    pub token_id: String,
    pub token_symbol: String,
    pub balance: GraphQLBalance,
    pub nonce: i32,
    pub receipt_chain_hash: String,
    pub delegate: Option<String>,
    pub voting_for: String,
    pub timing: GraphQLTiming,
    pub permissions: GraphQLPermissions,
    // pub zkapp: Option<GraphQLZkAppAccount>,
}

#[derive(GraphQLObject)]
pub struct GraphQLTiming {
    pub is_timed: bool,
    pub initial_minimum_balance: String,
    pub cliff_time: i32,
    pub cliff_amount: String,
    pub vesting_period: i32,
    pub vesting_increment: String,
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
    pub txn_version: i32,
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
    pub max_proofs_verified: String,
    pub actual_wrap_domain_size: String,
    pub wrap_index: String,
}

impl From<ledger::SetVerificationKey<ledger::AuthRequired>> for GraphQLSetVerificationKey {
    fn from(value: ledger::SetVerificationKey<ledger::AuthRequired>) -> Self {
        Self {
            auth: serde_json::to_string(&value.auth).unwrap(),
            txn_version: value.txn_version.as_u32() as i32,
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
                is_timed: false,
                initial_minimum_balance: "0".to_string(),
                vesting_period: 1,
                cliff_time: 0,
                cliff_amount: "0".to_string(),
                vesting_increment: "0".to_string(),
            },
            ledger::Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => Self {
                is_timed: true,
                initial_minimum_balance: initial_minimum_balance.as_u64().to_string(),
                cliff_time: cliff_time.as_u32() as i32,
                cliff_amount: cliff_amount.as_u64().to_string(),
                vesting_period: vesting_period.as_u32() as i32,
                vesting_increment: vesting_increment.as_u64().to_string(),
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
            token_id: TokenIdKeyHash::from(value.token_id).to_string(),
            token_symbol: value.token_symbol.0,
            balance: GraphQLBalance::from(value.balance),
            nonce: value.nonce.as_u32() as i32,
            receipt_chain_hash: ReceiptChainHash::from(value.receipt_chain_hash).to_string(),
            delegate: value.delegate.map(|d| d.into_address()),
            voting_for: value.voting_for.to_base58check(),
            timing: GraphQLTiming::from(value.timing),
            permissions: GraphQLPermissions::from(value.permissions),
            // zkapp: value.zkapp.map(GraphQLZkAppAccount::from), // Commented out as in the original
        }
    }
}

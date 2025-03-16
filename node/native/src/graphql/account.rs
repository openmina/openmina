use std::{collections::HashMap, sync::Arc};

use dataloader::non_cached::Loader;
use juniper::{graphql_object, FieldResult, GraphQLInputObject, GraphQLObject};
use ledger::{AccountId, FpExt};
use mina_p2p_messages::{
    string::{TokenSymbol, ZkAppUri},
    v2::{
        MinaBaseAccountUpdateUpdateTimingInfoStableV1, MinaBaseVerificationKeyWireStableV1,
        ReceiptChainHash, TokenIdKeyHash,
    },
};
use node::account::AccountPublicKey;
use openmina_node_common::rpc::RpcSender;

use super::{Context, ConversionError};

pub(crate) type AccountLoader =
    Loader<AccountPublicKey, Result<GraphQLAccount, Arc<ConversionError>>, AccountBatcher>;

pub(crate) struct AccountBatcher {
    rpc_sender: RpcSender,
}

impl dataloader::BatchFn<AccountPublicKey, Result<GraphQLAccount, Arc<ConversionError>>>
    for AccountBatcher
{
    async fn load(
        &mut self,
        keys: &[AccountPublicKey],
    ) -> HashMap<AccountPublicKey, Result<GraphQLAccount, Arc<ConversionError>>> {
        todo!()
    }
}

pub(crate) fn create_account_loader(rpc_sender: RpcSender) -> AccountLoader {
    // TODO(adonagy): is 25 enough?
    Loader::new(AccountBatcher { rpc_sender }).with_yield_count(25)
}

#[derive(Debug, Clone)]
pub(crate) struct GraphQLAccount {
    public_key: String,
    token_id: String,
    token: String,
    token_symbol: String,
    balance: GraphQLBalance,
    nonce: String,
    receipt_chain_hash: String,
    delegate_account: Option<Box<GraphQLAccount>>,
    // Storing the key for later
    delegate_key: Option<AccountPublicKey>,
    voting_for: String,
    timing: GraphQLTiming,
    permissions: GraphQLPermissions,
    // can we flatten?
    // pub zkapp: Option<GraphQLZkAppAccount>,
    zkapp_state: Option<Vec<String>>,
    verification_key: Option<GraphQLVerificationKey>,
    action_state: Option<Vec<String>>,
    proved_state: Option<bool>,
    zkapp_uri: Option<String>,
}

#[graphql_object(context = Context)]
#[graphql(description = "A Mina account")]
impl GraphQLAccount {
    fn public_key(&self) -> &str {
        &self.public_key
    }

    fn token_id(&self) -> &str {
        &self.token_id
    }

    fn token(&self) -> &str {
        &self.token
    }

    fn token_symbol(&self) -> &str {
        &self.token_symbol
    }

    fn balance(&self) -> &GraphQLBalance {
        &self.balance
    }

    fn nonce(&self) -> &str {
        &self.nonce
    }

    fn receipt_chain_hash(&self) -> &str {
        &self.receipt_chain_hash
    }

    async fn delegate_account(
        &self,
        context: &Context,
    ) -> FieldResult<Option<Box<GraphQLAccount>>> {
        // If we have a delegate key
        if let Some(delegate_key) = self.delegate_key.as_ref() {
            // Use the loader to fetch the delegate account
            let delegate_result = context
                .account_loader
                .try_load(delegate_key.clone())
                .await
                .map_err(|e| {
                    juniper::FieldError::new(
                        format!("Failed to load delegate account: {}", e),
                        juniper::Value::null(),
                    )
                })?;

            // Handle the result
            match delegate_result {
                Ok(account) => Ok(Some(Box::new(account))),
                Err(e) => Err(juniper::FieldError::new(
                    format!("Error loading delegate account: {}", e),
                    juniper::Value::null(),
                )),
            }
        } else {
            // No delegate
            Ok(None)
        }
    }

    fn voting_for(&self) -> &str {
        &self.voting_for
    }

    fn timing(&self) -> &GraphQLTiming {
        &self.timing
    }

    fn permissions(&self) -> &GraphQLPermissions {
        &self.permissions
    }

    fn zkapp_state(&self) -> &Option<Vec<String>> {
        &self.zkapp_state
    }

    fn verification_key(&self) -> &Option<GraphQLVerificationKey> {
        &self.verification_key
    }

    fn action_state(&self) -> &Option<Vec<String>> {
        &self.action_state
    }

    fn proved_state(&self) -> &Option<bool> {
        &self.proved_state
    }

    fn zkapp_uri(&self) -> &Option<String> {
        &self.zkapp_uri
    }
}

#[derive(GraphQLObject, Debug, Clone)]
pub struct GraphQLDelegateAccount {
    pub public_key: String,
}

#[derive(GraphQLObject, Debug, Clone)]
pub struct GraphQLTiming {
    // pub is_timed: bool,
    pub initial_minimum_balance: Option<String>,
    pub cliff_time: Option<i32>,
    pub cliff_amount: Option<String>,
    pub vesting_period: Option<i32>,
    pub vesting_increment: Option<String>,
}

#[derive(GraphQLInputObject, Debug, Clone)]
pub struct InputGraphQLTiming {
    // pub is_timed: bool,
    pub initial_minimum_balance: String,
    pub cliff_time: i32,
    pub cliff_amount: String,
    pub vesting_period: i32,
    pub vesting_increment: String,
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

#[derive(GraphQLObject, Debug, Clone)]
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

#[derive(GraphQLObject, Debug, Clone)]
pub struct GraphQLSetVerificationKey {
    pub auth: String,
    pub txn_version: String,
}

#[derive(GraphQLObject, Debug, Clone)]
pub struct GraphQLBalance {
    pub total: String,
}

// #[derive(GraphQLObject, Debug)]
// pub struct GraphQLZkAppAccount {
//     pub app_state: Vec<String>,
//     pub verification_key: Option<GraphQLVerificationKey>,
//     pub zkapp_version: i32,
//     pub action_state: Vec<String>,
//     pub last_action_slot: i32,
//     pub proved_state: bool,
//     pub zkapp_uri: String,
// }

#[derive(GraphQLObject, Debug, Clone)]
pub struct GraphQLVerificationKey {
    // pub max_proofs_verified: String,
    // pub actual_wrap_domain_size: String,
    // pub wrap_index: String,
    pub verification_key: String,
    pub hash: String,
}

#[derive(GraphQLObject, Debug, Clone)]
/// Dummy type to represent [`GraphQLAccount`]
pub struct GraphQLDummyAccount {
    pub public_key: String,
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

impl TryFrom<ledger::Account> for GraphQLAccount {
    type Error = ConversionError;

    fn try_from(value: ledger::Account) -> Result<Self, Self::Error> {
        // Process the verification_key with proper error handling
        let verification_key = value
            .zkapp
            .clone()
            .and_then(|zkapp| {
                zkapp.verification_key.map(|vk| {
                    let ser = MinaBaseVerificationKeyWireStableV1::from(vk.vk()).to_base64()?;

                    Ok(GraphQLVerificationKey {
                        verification_key: ser,
                        hash: vk.hash().to_decimal(),
                    }) as Result<GraphQLVerificationKey, Self::Error>
                })
            })
            .transpose()?; // Transpose Option<Result<...>> to Result<Option<...>>

        Ok(Self {
            public_key: value.public_key.into_address(),
            token_id: TokenIdKeyHash::from(value.token_id.clone()).to_string(),
            token: TokenIdKeyHash::from(value.token_id).to_string(),
            token_symbol: TokenSymbol::from(&value.token_symbol).to_string(),
            balance: GraphQLBalance::from(value.balance),
            nonce: value.nonce.as_u32().to_string(),
            receipt_chain_hash: ReceiptChainHash::from(value.receipt_chain_hash).to_string(),
            delegate_key: value.delegate.map(AccountPublicKey::from),
            // Initialy set to None, will be set in the resolver
            delegate_account: None,
            voting_for: value.voting_for.to_base58check_graphql(),
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
            verification_key,
            action_state: value.zkapp.clone().map(|zkapp| {
                zkapp
                    .action_state
                    .into_iter()
                    .map(|v| v.to_decimal())
                    .collect::<Vec<_>>()
            }),
            proved_state: value.zkapp.clone().map(|zkapp| zkapp.proved_state),
            zkapp_uri: value
                .zkapp
                .map(|zkapp| ZkAppUri::from(&zkapp.zkapp_uri).to_string()),
        })
    }
}

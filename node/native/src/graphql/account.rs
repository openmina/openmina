//! Implements GraphQL types and resolvers for Mina accounts, providing access to account balances, permissions, and zkApp state.
//! This module includes data loaders for efficient batched account queries and conversion logic between internal and GraphQL types.

use std::{collections::HashMap, sync::Arc};

use dataloader::non_cached::Loader;
use juniper::{graphql_object, FieldResult, GraphQLInputObject, GraphQLObject};
use ledger::{
    scan_state::currency::{Balance, Magnitude, Slot},
    Account, AccountId, FpExt, Timing,
};
use mina_p2p_messages::{
    string::{TokenSymbol, ZkAppUri},
    v2::{
        MinaBaseAccountUpdateUpdateTimingInfoStableV1, MinaBaseVerificationKeyWireStableV1,
        ReceiptChainHash, TokenIdKeyHash,
    },
};
use mina_signer::CompressedPubKey;
use node::rpc::{AccountQuery, RpcRequest};
use openmina_node_common::rpc::RpcSender;

use super::{Context, ConversionError};

pub(crate) type AccountLoader =
    Loader<AccountId, Result<GraphQLAccount, Arc<ConversionError>>, AccountBatcher>;

pub(crate) struct AccountBatcher {
    rpc_sender: RpcSender,
}

impl dataloader::BatchFn<AccountId, Result<GraphQLAccount, Arc<ConversionError>>>
    for AccountBatcher
{
    async fn load(
        &mut self,
        keys: &[AccountId],
    ) -> HashMap<AccountId, Result<GraphQLAccount, Arc<ConversionError>>> {
        self.rpc_sender
            .oneshot_request::<Vec<Account>>(RpcRequest::LedgerAccountsGet(
                AccountQuery::MultipleIds(keys.to_vec()),
            ))
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|account| (account.id(), account.try_into().map_err(Arc::new)))
            .collect()
    }
}

pub(crate) fn create_account_loader(rpc_sender: RpcSender) -> AccountLoader {
    // TODO(adonagy): is 25 enough?
    Loader::new(AccountBatcher { rpc_sender }).with_yield_count(25)
}

#[derive(Debug, Clone)]
pub(crate) struct GraphQLAccount {
    inner: Account,
    public_key: String,
    token_id: String,
    token: String,
    token_symbol: String,
    // balance: GraphQLBalance,
    nonce: String,
    receipt_chain_hash: String,
    // Storing the key for later
    delegate_key: Option<CompressedPubKey>,
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

impl GraphQLAccount {
    /// Calculates the minimum balance required for this account at the given global slot.
    ///
    /// For timed accounts, this represents the locked portion of funds based on vesting schedule.
    /// For untimed accounts, this is always zero.
    fn min_balance(&self, global_slot: Option<u32>) -> Option<Balance> {
        global_slot.map(|slot| match self.inner.timing {
            Timing::Untimed => Balance::zero(),
            Timing::Timed { .. } => self.inner.min_balance_at_slot(Slot::from_u32(slot)),
        })
    }

    /// Calculates the liquid (spendable) balance for this account at the given global slot.
    ///
    /// This is the difference between total balance and minimum balance (locked funds).
    /// If total balance is less than minimum balance, returns zero.
    fn liquid_balance(&self, global_slot: Option<u32>) -> Option<Balance> {
        let min_balance = self.min_balance(global_slot);
        let total = self.inner.balance;
        min_balance.map(|mb| {
            if total > mb {
                total.checked_sub(&mb).expect("overflow")
            } else {
                Balance::zero()
            }
        })
    }
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

    async fn balance(&self, context: &Context) -> GraphQLBalance {
        let best_tip = context.get_or_fetch_best_tip().await;
        let global_slot = best_tip.as_ref().map(|bt| bt.global_slot());

        GraphQLBalance {
            total: self.inner.balance.as_u64().to_string(),
            block_height: best_tip
                .as_ref()
                .map(|bt| bt.height())
                .unwrap_or_default()
                .to_string(),
            state_hash: best_tip.as_ref().map(|bt| bt.hash().to_string()),
            liquid: self
                .liquid_balance(global_slot)
                .map(|b| b.as_u64().to_string()),
            locked: self
                .min_balance(global_slot)
                .map(|b| b.as_u64().to_string()),
            unknown: self.inner.balance.as_u64().to_string(),
        }
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
            // A delegate always has the default token id
            let delegate_id = AccountId::new_with_default_token(delegate_key.clone());
            // Use the loader to fetch the delegate account
            Ok(context.load_account(delegate_id).await.map(Box::new))
        } else {
            // No delegate
            Ok(None)
        }
    }

    pub async fn delegators(&self, context: &Context) -> FieldResult<Vec<GraphQLAccount>> {
        if let Some(best_tip) = context.get_or_fetch_best_tip().await {
            let staking_ledger_hash = best_tip.staking_epoch_ledger_hash();

            let id = self.inner.id();
            let delegators = context
                .fetch_delegators(staking_ledger_hash.clone(), id.clone())
                .await
                .unwrap_or_default();

            Ok(delegators
                .into_iter()
                .map(GraphQLAccount::try_from)
                .collect::<Result<Vec<_>, _>>()?)
        } else {
            Ok(vec![])
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
    pub block_height: String,
    pub state_hash: Option<String>,
    pub liquid: Option<String>,
    pub locked: Option<String>,
    pub unknown: String,
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

impl TryFrom<ledger::Account> for GraphQLAccount {
    type Error = ConversionError;

    /// Converts a ledger Account to a GraphQL-compatible account representation.
    ///
    /// This complex conversion handles all account fields including zkApp-specific data,
    /// with special attention to verification keys that require additional processing.
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
            inner: value.clone(),
            public_key: value.public_key.into_address(),
            token_id: TokenIdKeyHash::from(value.token_id.clone()).to_string(),
            token: TokenIdKeyHash::from(value.token_id).to_string(),
            token_symbol: TokenSymbol::from(&value.token_symbol).to_string(),
            // balance: GraphQLBalance::from(value.balance),
            nonce: value.nonce.as_u32().to_string(),
            receipt_chain_hash: ReceiptChainHash::from(value.receipt_chain_hash).to_string(),
            delegate_key: value.delegate,
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

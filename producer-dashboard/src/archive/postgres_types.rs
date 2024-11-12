use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "chain_status_type", rename_all = "snake_case")]
pub enum ChainStatus {
    Canonical,
    Orphaned,
    Pending,
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "authorization_kind_type")]
pub enum AuthorizationKind {
    #[sqlx(rename = "None_given")]
    NoneGiven,
    Signature,
    Proof,
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "transaction_status_type", rename_all = "snake_case")]
pub enum TransactionStatus {
    Applied,
    Failed,
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "internal_command_type", rename_all = "snake_case")]
pub enum InternalCommandType {
    FeeTransferViaCoinbase,
    FeeTransfer,
    Coinbase,
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "may_use_token")]
pub enum MayUseToken {
    No,
    ParentsOwnToken,
    InheritFromParent,
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "user_command_type", rename_all = "snake_case")]
pub enum UserCommandType {
    Payment,
    Delegation,
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "zkapp_auth_required_type", rename_all = "snake_case")]
pub enum ZkappAuthRequiredType {
    None,
    Either,
    Proof,
    Signature,
    Both,
    Impossible,
}

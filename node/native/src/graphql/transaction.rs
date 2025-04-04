//! Implements GraphQL types for transaction status information.
//! This module provides enums and conversion logic for transaction status queries.

use juniper::GraphQLEnum;
use node::rpc::TransactionStatus;

#[derive(Clone, Copy, Debug, GraphQLEnum)]
#[allow(non_camel_case_types)]
pub enum GraphQLTransactionStatus {
    INCLUDED,
    PENDING,
    UNKNOWN,
}

impl From<TransactionStatus> for GraphQLTransactionStatus {
    fn from(value: TransactionStatus) -> Self {
        match value {
            TransactionStatus::Included => Self::INCLUDED,
            TransactionStatus::Pending => Self::PENDING,
            TransactionStatus::Unknown => Self::UNKNOWN,
        }
    }
}

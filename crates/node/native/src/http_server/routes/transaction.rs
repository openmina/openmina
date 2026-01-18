//! Transaction endpoints.
//!
//! - `GET /transaction-pool` - Get transaction pool
//! - `GET /accounts` - Get all accounts
//! - `POST /send-payment` - Send a payment transaction
//! - `GET /best-chain-user-commands` - Get user commands from best chain

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use mina_node::rpc::{
    RpcInjectPayment, RpcLedgerSlimAccountsResponse, RpcTransactionInjectResponse,
    RpcTransactionPoolResponse, RpcTransitionFrontierUserCommandsResponse,
};

use crate::http_server::{AppError, AppResult, AppState};

/// Registers transaction routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/transaction-pool", get(transaction_pool))
        .route("/accounts", get(accounts))
        .route("/send-payment", post(send_payment))
        .route("/best-chain-user-commands", get(best_chain_user_commands))
}

/// Returns the transaction pool.
async fn transaction_pool(
    State(state): State<AppState>,
) -> AppResult<Json<RpcTransactionPoolResponse>> {
    state
        .rpc_sender()
        .transaction_pool()
        .get()
        .await
        .map(Json)
        .ok_or(AppError::ChannelDropped)
}

/// Returns all accounts from the latest ledger.
async fn accounts(State(state): State<AppState>) -> AppResult<Json<RpcLedgerSlimAccountsResponse>> {
    state
        .rpc_sender()
        .ledger()
        .latest()
        .accounts()
        .all()
        .await
        .map(Json)
        .ok_or(AppError::ChannelDropped)
}

/// Sends payment transactions.
async fn send_payment(
    State(state): State<AppState>,
    Json(payments): Json<Vec<RpcInjectPayment>>,
) -> AppResult<Json<RpcTransactionInjectResponse>> {
    match state
        .rpc_sender()
        .transaction_pool()
        .inject()
        .payment(payments)
        .await
    {
        Err(err) => Err(AppError::Internal(err)),
        Ok(None) => Err(AppError::ChannelDropped),
        Ok(Some(resp)) => Ok(Json(resp)),
    }
}

/// Returns user commands from the best chain.
async fn best_chain_user_commands(
    State(state): State<AppState>,
) -> AppResult<Json<RpcTransitionFrontierUserCommandsResponse>> {
    state
        .rpc_sender()
        .transition_frontier()
        .best_chain()
        .user_commands()
        .await
        .map(Json)
        .ok_or(AppError::ChannelDropped)
}

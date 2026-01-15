//! Transaction endpoints.
//!
//! - `GET /transaction-pool` - Get transaction pool
//! - `GET /accounts` - Get all accounts
//! - `POST /send-payment` - Send a payment transaction
//! - `GET /best-chain-user-commands` - Get user commands from best chain

use axum::Router;

use crate::http_server::AppState;

/// Registers transaction routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    // TODO: Implement handlers
    router
}

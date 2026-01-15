//! State inspection endpoints.
//!
//! - `GET /state` - Get node state with optional filter
//! - `POST /state` - Get node state with filter in body
//! - `GET /state/peers` - Get connected peers
//! - `GET /state/message-progress` - Get message progress

use axum::Router;

use crate::http_server::AppState;

/// Registers state routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    // TODO: Implement handlers
    router
}

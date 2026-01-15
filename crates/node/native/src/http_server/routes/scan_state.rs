//! Scan state endpoints.
//!
//! - `GET /scan-state/summary` - Get scan state summary for best tip
//! - `GET /scan-state/summary/{block}` - Get scan state summary for specific block

use axum::Router;

use crate::http_server::AppState;

/// Registers scan state routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    // TODO: Implement handlers
    router
}

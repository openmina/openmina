//! Discovery endpoints.
//!
//! - `GET /discovery/routing_table` - Get Kademlia routing table
//! - `GET /discovery/bootstrap_stats` - Get bootstrap statistics

use axum::Router;

use crate::http_server::AppState;

/// Registers discovery routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    // TODO: Implement handlers
    router
}

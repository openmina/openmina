//! Snark pool endpoints.
//!
//! - `GET /snark-pool/jobs` - Get all snark pool jobs
//! - `GET /snark-pool/job/{id}` - Get specific snark pool job

use axum::Router;

use crate::http_server::AppState;

/// Registers snark pool routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    // TODO: Implement handlers
    router
}

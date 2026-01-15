//! Snarker endpoints.
//!
//! - `POST /snarker/job/commit` - Commit to a snark job
//! - `GET /snarker/job/spec` - Get snark job specification
//! - `GET /snarker/workers` - Get snarker workers
//! - `GET /snarker/config` - Get snarker configuration

use axum::Router;

use crate::http_server::AppState;

/// Registers snarker routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    // TODO: Implement handlers
    router
}

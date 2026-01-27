//! Route handlers for the axum HTTP server.
//!
//! Each submodule groups related endpoints by functionality.

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use super::{openapi, AppState};

pub mod discovery;
pub mod graphql;
pub mod scan_state;
pub mod snark_pool;
pub mod snarker;
pub mod state;
pub mod stats;
pub mod status;
pub mod transaction;

#[cfg(feature = "p2p-webrtc")]
pub mod webrtc;

/// Builds the OpenAPI router with all documented routes.
///
/// This is used by both the HTTP server and for generating the OpenAPI spec.
pub fn openapi_router() -> OpenApiRouter<AppState> {
    let router = OpenApiRouter::with_openapi(openapi::ApiDoc::openapi())
        .merge(status::routes())
        .merge(state::routes())
        .merge(stats::routes())
        .merge(scan_state::routes())
        .merge(snark_pool::routes())
        .merge(snarker::routes())
        .merge(transaction::routes())
        .merge(discovery::routes());

    #[cfg(feature = "p2p-webrtc")]
    let router = router.merge(webrtc::routes());

    router
}

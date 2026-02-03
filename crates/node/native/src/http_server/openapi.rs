//! OpenAPI documentation configuration.

use utoipa::OpenApi;

/// Base OpenAPI documentation with API metadata.
///
/// Tags and paths are registered via `utoipa-axum`'s `OpenApiRouter`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Mina Node HTTP RPC",
        version = env!("CARGO_PKG_VERSION"),
        description = "HTTP RPC API for Mina Rust node status, state, and operations"
    ),
    servers(
        (url = "http://localhost:3000", description = "Default port of local node")
    ),
    tags(
        (name = "status", description = "Node health and status"),
        (name = "kubernetes", description = "Kubernetes probe endpoints"),
        (name = "state", description = "Node state inspection"),
        (name = "stats", description = "Statistics and metrics"),
        (name = "scan-state", description = "Scan state inspection"),
        (name = "snark-pool", description = "SNARK pool management"),
        (name = "snarker", description = "SNARK worker operations"),
        (name = "transaction", description = "Transaction pool and accounts"),
        (name = "discovery", description = "P2P discovery info"),
        (name = "webrtc", description = "WebRTC signaling"),
    )
)]
pub struct ApiDoc;

/// Stoplight Elements UI (CDN-loaded)
#[cfg(feature = "stoplight-elements")]
pub async fn stoplight_elements() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("stoplight_elements.html"))
}

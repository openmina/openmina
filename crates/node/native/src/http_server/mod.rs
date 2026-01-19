//! Axum-based HTTP server for the Mina node RPC API.
//!
//! This module provides REST endpoints for node status, state inspection,
//! snark pool management, and transaction handling.

#[macro_use]
mod macros;
mod openapi;
mod routes;
mod types;

pub use types::{AppError, AppResult, AppState};

use mina_node_common::rpc::RpcSender;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use types::cors_layer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::SwaggerUi;

#[cfg(feature = "scalar")]
use utoipa_scalar::{Scalar, Servable};

/// Runs the HTTP server on the specified port.
///
/// Returns an error if binding to the port fails or the server encounters an I/O error.
pub async fn run(port: u16, rpc_sender: RpcSender) -> std::io::Result<()> {
    let state = AppState::new(rpc_sender);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            tracing::debug_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
            )
        })
        .on_response(
            |response: &axum::http::Response<_>, latency, _span: &tracing::Span| {
                let status = response.status();
                if status.is_server_error() || status.is_client_error() {
                    tracing::error!(status = %status, latency = ?latency, "request failed");
                } else {
                    tracing::info!(status = %status, latency = ?latency, "request completed");
                }
            },
        );

    // Build OpenApiRouter with documented routes
    let openapi_router = OpenApiRouter::with_openapi(openapi::ApiDoc::openapi())
        .merge(routes::status::routes())
        .merge(routes::state::routes())
        .merge(routes::stats::routes())
        .merge(routes::scan_state::routes())
        .merge(routes::snark_pool::routes())
        .merge(routes::snarker::routes())
        .merge(routes::transaction::routes())
        .merge(routes::discovery::routes());

    #[cfg(feature = "p2p-webrtc")]
    let openapi_router = openapi_router.merge(routes::webrtc::routes());

    // Split to get Router and OpenApi spec
    let (app, api) = openapi_router.split_for_parts();

    // GraphQL (not documented in OpenAPI)
    let app = routes::graphql::routes(app);

    // OpenAPI documentation UIs
    #[cfg(feature = "swagger-ui")]
    let app = app.merge(SwaggerUi::new("/api-docs/swagger-ui").url("/api-docs/openapi.json", api.clone()));

    #[cfg(feature = "scalar")]
    let app = app.merge(Scalar::with_url("/api-docs/scalar", api));

    #[cfg(feature = "stoplight-elements")]
    let app = app.route("/api-docs/stoplight", axum::routing::get(openapi::stoplight_elements));

    let app = app.layer(trace_layer).layer(cors_layer()).with_state(state);

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "HTTP server listening");
    axum::serve(listener, app).await
}

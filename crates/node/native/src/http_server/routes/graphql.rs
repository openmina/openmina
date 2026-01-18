//! GraphQL endpoints.
//!
//! - `POST /graphql` - Execute GraphQL queries and mutations
//! - `GET /graphiql` - GraphiQL IDE (requires `graphiql` feature, on by default)
//!
//! Note: `/playground` is intentionally not included. GraphQL Playground was
//! deprecated and merged into GraphiQL 2.0, making them redundant.

use std::sync::Arc;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Json, Router,
};
use juniper::{http::GraphQLBatchRequest, EmptySubscription, RootNode};

use crate::{
    graphql::{Context, Mutation, Query},
    http_server::AppState,
};

type Schema = RootNode<Query, Mutation, EmptySubscription<Context>>;

/// Registers GraphQL routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    let schema = Arc::new(Schema::new(Query, Mutation, EmptySubscription::new()));

    let router = router.route("/graphql", post(graphql_handler));

    #[cfg(feature = "graphiql")]
    let router = router.route("/graphiql", get(graphiql_handler));

    router.layer(Extension(schema))
}

/// Handles GraphQL POST requests.
async fn graphql_handler(
    State(state): State<AppState>,
    Extension(schema): Extension<Arc<Schema>>,
    Json(request): Json<GraphQLBatchRequest>,
) -> impl IntoResponse {
    let context = Context::new(state.rpc_sender().clone());
    let response = request.execute(&*schema, &context).await;
    Json(response)
}

/// Serves the GraphiQL IDE.
#[cfg(feature = "graphiql")]
async fn graphiql_handler() -> impl IntoResponse {
    Html(juniper::http::graphiql::graphiql_source("/graphql", None))
}

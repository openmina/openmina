//! Macros for HTTP handlers.
//!
//! These macros simplify the common pattern of sending an RPC request and
//! converting the response to an HTTP response.
//!
//! # Type flow
//!
//! ```text
//! oneshot_request(req)  -> Option<T>
//! rpc_request!(...)     -> AppResult<T>      (= Result<T, AppError>)
//! jsonify_rpc!(...)     -> AppResult<Json<T>> (= Result<Json<T>, AppError>)
//! ```
//!
//! The `Option<T>` from `oneshot_request` represents channel status:
//! - `Some(T)` - response received
//! - `None` - channel dropped (converted to `AppError::ChannelDropped`)
//!
//! Note: `T` is often itself an `Option<U>`, where `None` means "no data
//! available" (serialized as `null`). This is distinct from channel failure.

/// Sends an RPC request, returning an error if the channel was dropped.
///
/// Returns `Result<T, AppError>` where `T` is the RPC response type.
#[macro_export]
macro_rules! rpc_request {
    ($state:expr, $request:expr) => {
        $state
            .rpc_sender()
            .oneshot_request($request)
            .await
            .ok_or($crate::http_server::AppError::ChannelDropped)
    };
}

/// Sends an RPC request and wraps the response in JSON.
///
/// Returns `Result<Json<T>, AppError>` where `T` is the RPC response type.
#[macro_export]
macro_rules! jsonify_rpc {
    ($state:expr, $request:expr) => {
        rpc_request!($state, $request).map(axum::Json)
    };
}

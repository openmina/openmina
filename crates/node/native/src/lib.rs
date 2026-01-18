//! # Native Node Library
//!
//! Platform-specific implementation of the Mina node for native targets
//! (Linux, macOS, Windows).
//!
//! This crate provides:
//! - [`NodeBuilder`] - Fluent API for constructing nodes
//! - HTTP server (axum-based) with REST API and GraphQL
//! - [`NodeService`] - Native service implementations (P2P, ledger, proofs)
//!
//! Used by the CLI (`cli` crate) to instantiate and run nodes with RPC servers.

pub use mina_node_common::*;

pub mod graphql;
pub mod http_server;

mod service;
pub use service::{NodeService, *};

mod node;
pub use node::{Node, NodeBuilder};

#[path = "replay.rs"]
mod replayer;
pub use replayer::*;

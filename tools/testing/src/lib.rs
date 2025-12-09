//! # Mina Node Testing Framework
//!
//! A comprehensive testing framework for the Mina Rust node implementation.
//! Provides scenario-based testing capabilities with deterministic execution,
//! cluster management, and cross-implementation compatibility testing.
//!
//! ## Key Features
//!
//! - **Scenario-based testing**: Deterministic, repeatable test sequences
//! - **Cluster orchestration**: Multi-node test coordination
//! - **Cross-implementation**: Tests both Rust and OCaml node compatibility
//! - **Recording/replay**: Capture and reproduce test scenarios
//! - **Network simulation**: Controlled network environments
//!
//! ## Documentation
//!
//! For detailed usage and examples, see:
//! - [Scenario Tests](https://o1-labs.github.io/mina-rust/developers/testing/scenario-tests)
//! - [Testing Framework](https://o1-labs.github.io/mina-rust/developers/testing/testing-framework)
//! - [Network Connectivity](https://o1-labs.github.io/mina-rust/developers/testing/network-connectivity)
//!
//! ## Usage
//!
//! The main entry point is the `mina-node-testing` binary:
//!
//! ```bash
//! # List available scenarios
//! cargo run --release --bin mina-node-testing -- scenarios-list
//!
//! # Run a specific scenario
//! cargo run --release --bin mina-node-testing -- scenarios-run --name scenario-name
//! ```

mod exit_with_error;

use std::sync::Arc;

pub use exit_with_error::exit_with_error;

pub mod cluster;
pub mod node;
pub mod scenario;
#[cfg(feature = "scenario-generators")]
pub mod scenarios;
pub mod service;
pub mod simulator;

pub mod hosts;
pub mod network_debugger;

mod server;
pub use server::server;
use tokio::sync::{Mutex, MutexGuard};

pub fn setup() -> tokio::runtime::Runtime {
    mina_node_native::tracing::initialize(mina_node_native::tracing::Level::INFO);
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get().max(2) - 1)
        .thread_name(|i| format!("mina_rayon_{i}"))
        .build_global()
        .unwrap();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

pub fn setup_without_rt() {
    lazy_static::lazy_static! {
        static ref INIT: () = {
            let level = std::env::var("MINA_TRACING_LEVEL").ok().and_then(|level| {
                match level.parse() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        eprintln!("cannot parse {level} as tracing level: {e}");
                        None
                    }
                }
            }).unwrap_or(mina_node_native::tracing::Level::INFO);
            mina_node_native::tracing::initialize(level);

            if let Err(err) = tracing_log::LogTracer::init() {
                eprintln!("cannot initialize log tracing bridge: {err}");
            }

            rayon::ThreadPoolBuilder::new()
                .num_threads(num_cpus::get().max(2) - 1)
                .thread_name(|i| format!("mina_rayon_{i}"))
                .build_global()
                .unwrap();
        };
    };
    *INIT;
}

lazy_static::lazy_static! {
    static ref GATE: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

pub struct TestGate(#[allow(dead_code)] MutexGuard<'static, ()>);

impl TestGate {
    async fn there_can_be_only_one() -> Self {
        Self(GATE.lock().await)
    }
    pub fn release(self) {}
}

pub async fn wait_for_other_tests() -> TestGate {
    TestGate::there_can_be_only_one().await
}

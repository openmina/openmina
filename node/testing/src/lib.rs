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

pub mod network_debugger;

mod server;
pub use server::server;
use tokio::sync::{Mutex, MutexGuard};

pub fn setup() -> tokio::runtime::Runtime {
    openmina_node_native::tracing::initialize(openmina_node_native::tracing::Level::INFO);
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get().max(2) - 1)
        .thread_name(|i| format!("openmina_rayon_{i}"))
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
            let level = std::env::var("OPENMINA_TRACING_LEVEL").ok().and_then(|level| {
                match level.parse() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        eprintln!("cannot parse {level} as tracing level: {e}");
                        None
                    }
                }
            }).unwrap_or(openmina_node_native::tracing::Level::INFO);
            openmina_node_native::tracing::initialize(level);

            if let Err(err) = tracing_log::LogTracer::init() {
                eprintln!("cannot initialize log tracing bridge: {err}");
            }

            rayon::ThreadPoolBuilder::new()
                .num_threads(num_cpus::get().max(2) - 1)
                .thread_name(|i| format!("openmina_rayon_{i}"))
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

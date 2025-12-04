//! # Mina Node Testing CLI
//!
//! Command-line interface for running Mina node scenario tests.
//! Provides tools for generating, running, and managing deterministic
//! blockchain testing scenarios.
//!
//! ## Documentation
//!
//! For detailed documentation and usage examples, see:
//! - [Scenario Tests](https://o1-labs.github.io/mina-rust/developers/testing/scenario-tests) - Complete testing guide
//! - [Testing Framework](https://o1-labs.github.io/mina-rust/developers/testing/testing-framework) - Testing architecture
//!
//! ## Quick Start
//!
//! List all available scenarios:
//! ```bash
//! cargo run --release --bin mina-node-testing -- scenarios-list
//! ```
//!
//! Run a specific scenario:
//! ```bash
//! cargo run --release --bin mina-node-testing -- scenarios-run --name p2p-signaling
//! ```

use clap::Parser;

use mina_node_testing::{
    cluster::{Cluster, ClusterConfig},
    exit_with_error,
    scenario::Scenario,
    scenarios::Scenarios,
    server, setup,
};
use node::p2p::webrtc::Host;

pub type CommandError = anyhow::Error;

#[derive(Debug, clap::Parser)]
#[command(name = "mina-testing", about = "Mina Testing Cli")]
pub struct MinaTestingCli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Server(CommandServer),

    ScenariosGenerate(CommandScenariosGenerate),
    ScenariosRun(CommandScenariosRun),
    ScenariosList(CommandScenariosList),
}

#[derive(Debug, clap::Args)]
pub struct CommandServer {
    #[arg(long, short, default_value = "127.0.0.1")]
    pub host: Host,

    #[arg(long, short, default_value = "11000")]
    pub port: u16,
    #[arg(long, short)]
    pub ssl_port: Option<u16>,
}

#[derive(Debug, clap::Args)]
pub struct CommandScenariosGenerate {
    #[arg(long, short)]
    pub name: Option<String>,
    #[arg(long, short)]
    pub use_debugger: bool,
    #[arg(long, short)]
    pub webrtc: bool,
    #[arg(long, short = 'o', default_value = "stdout", value_enum)]
    pub output: OutputFormat,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Stdout,
    Json,
}

/// Run scenario located at `res/scenarios`.
#[derive(Debug, clap::Args)]
pub struct CommandScenariosRun {
    /// Name of the scenario.
    ///
    /// Must match filename in `res/scenarios` (without an extension).
    #[arg(long, short)]
    pub name: String,
}

#[derive(Debug, clap::Args)]
pub struct CommandScenariosList {}

impl Command {
    pub fn run(self) -> Result<(), crate::CommandError> {
        let rt = setup();
        let _rt_guard = rt.enter();

        let (shutdown_tx, shutdown_rx) = mina_core::channels::oneshot::channel();
        let mut shutdown_tx = Some(shutdown_tx);

        ctrlc::set_handler(move || match shutdown_tx.take() {
            Some(tx) => {
                let _ = tx.send(());
            }
            None => {
                std::process::exit(1);
            }
        })
        .expect("Error setting Ctrl-C handler");

        match self {
            Self::Server(args) => {
                server(rt, args.host, args.port, args.ssl_port);
                Ok(())
            }
            Self::ScenariosGenerate(cmd) => {
                #[cfg(feature = "scenario-generators")]
                {
                    let output_format = cmd.output.clone();
                    let run_scenario = |scenario: Scenarios| -> Result<_, anyhow::Error> {
                        let mut config = scenario.default_cluster_config()?;
                        if cmd.use_debugger {
                            config.use_debugger();
                        }
                        if cmd.webrtc {
                            config.set_all_rust_to_rust_use_webrtc();
                        }
                        Ok((scenario, config))
                    };
                    let fut = async move {
                        if let Some(name) = cmd.name {
                            if let Some(scenario) = Scenarios::find_by_name(&name) {
                                let (scenario, config) = run_scenario(scenario)?;
                                match output_format {
                                    OutputFormat::Json => {
                                        scenario.run_and_save_from_scratch(config).await
                                    }
                                    OutputFormat::Stdout => {
                                        scenario.run_only_from_scratch(config).await
                                    }
                                }
                            } else {
                                anyhow::bail!("no such scenario: \"{name}\"");
                            }
                        } else {
                            for scenario in Scenarios::iter() {
                                let (scenario, config) = run_scenario(scenario)?;
                                match output_format {
                                    OutputFormat::Json => {
                                        scenario.run_and_save_from_scratch(config).await
                                    }
                                    OutputFormat::Stdout => {
                                        scenario.run_only_from_scratch(config).await
                                    }
                                }
                            }
                        }
                        Ok(())
                    };
                    rt.block_on(async {
                        tokio::select! {
                            res = fut => res,
                            _ = shutdown_rx => {
                                anyhow::bail!("Received ctrl-c signal! shutting down...");
                            }
                        }
                    })
                }
                #[cfg(not(feature = "scenario-generators"))]
                Err("binary not compiled with `scenario-generators` feature"
                    .to_owned()
                    .into())
            }
            Self::ScenariosRun(cmd) => {
                let mut config = ClusterConfig::new(None).map_err(|err| {
                    anyhow::anyhow!("failed to create cluster configuration: {err}")
                })?;
                config.set_replay();

                let id = cmd.name.parse()?;
                let fut = async move {
                    let mut cluster = Cluster::new(config);
                    cluster.start(Scenario::load(&id).await?).await?;
                    cluster.exec_to_end().await?;
                    for (node_id, node) in cluster.nodes_iter() {
                        let Some(best_tip) = node.state().transition_frontier.best_tip() else {
                            continue;
                        };

                        eprintln!(
                            "[node_status] node_{node_id} {} - {} [{}]",
                            best_tip.height(),
                            best_tip.hash(),
                            best_tip.producer()
                        );
                    }
                    Ok(())
                };
                rt.block_on(async {
                    tokio::select! {
                        res = fut => res,
                        _ = shutdown_rx => {
                            anyhow::bail!("Received ctrl-c signal! shutting down...");
                        }
                    }
                })
            }
            Self::ScenariosList(_) => {
                println!("Available scenarios:");
                for scenario in Scenarios::iter() {
                    println!("  {}", scenario.to_str())
                }
                Ok(())
            }
        }
    }
}

pub fn main() {
    match MinaTestingCli::parse().command.run() {
        Ok(_) => {}
        Err(err) => exit_with_error(err),
    }
}

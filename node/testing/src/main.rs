use clap::Parser;

use openmina_node_testing::cluster::{Cluster, ClusterConfig};
use openmina_node_testing::scenario::Scenario;
use openmina_node_testing::scenarios::Scenarios;
use openmina_node_testing::{exit_with_error, server, setup};

pub type CommandError = anyhow::Error;

#[derive(Debug, clap::Parser)]
#[command(name = "openmina-testing", about = "Openmina Testing Cli")]
pub struct OpenminaTestingCli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Server(CommandServer),

    ScenariosGenerate(CommandScenariosGenerate),
    ScenariosRun(CommandScenariosRun),
}

#[derive(Debug, clap::Args)]
pub struct CommandServer {
    #[arg(long, short, env, default_value = "11000")]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
pub struct CommandScenariosGenerate {
    #[arg(long, short)]
    pub name: Option<String>,
    #[arg(long, short)]
    pub use_debugger: bool,
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

impl Command {
    pub fn run(self) -> Result<(), crate::CommandError> {
        let rt = setup();
        let _rt_guard = rt.enter();

        let (shutdown_tx, shutdown_rx) = openmina_core::channels::oneshot::channel();
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
                server(args.port);
                Ok(())
            }
            Self::ScenariosGenerate(cmd) => {
                #[cfg(feature = "scenario-generators")]
                {
                    let config = ClusterConfig::new(None).map_err(|err| {
                        anyhow::anyhow!("failed to create cluster configuration: {err}")
                    })?;
                    let config = if cmd.use_debugger {
                        config.use_debugger()
                    } else {
                        config
                    };

                    let fut = async move {
                        if let Some(name) = cmd.name {
                            if let Some(scenario) = Scenarios::iter()
                                .into_iter()
                                .find(|s| <&'static str>::from(s) == name)
                            {
                                scenario.run_and_save_from_scratch(config).await;
                            } else {
                                anyhow::bail!("no such scenario: \"{name}\"");
                            }
                        } else {
                            for scenario in Scenarios::iter() {
                                scenario.run_and_save_from_scratch(config.clone()).await;
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
                let config = ClusterConfig::new(None).map_err(|err| {
                    anyhow::anyhow!("failed to create cluster configuration: {err}")
                })?;
                let config = config.set_replay();

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
        }
    }
}

pub fn main() {
    use std::alloc::System;

    use openmina_node_native::AllocTracker;
    use tracking_allocator::{AllocationGroupToken, AllocationRegistry, Allocator};

    #[global_allocator]
    static GLOBAL: Allocator<System> = Allocator::system();

    AllocationRegistry::set_global_tracker(AllocTracker::void())
        .expect("no other global tracker should be set yet");
    AllocationRegistry::enable_tracking();

    let mut local_token =
        AllocationGroupToken::register().expect("failed to register allocation group");
    let local_guard = local_token.enter();

    match OpenminaTestingCli::parse().command.run() {
        Ok(_) => {}
        Err(err) => exit_with_error(err),
    }

    drop(local_guard);
    AllocationRegistry::disable_tracking();
}

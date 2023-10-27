use clap::Parser;

use openmina_node_testing::{exit_with_error, server, setup};

pub type CommandError = Box<dyn std::error::Error>;

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
}

#[derive(Debug, clap::Args)]
pub struct CommandServer {
    #[arg(long, short, env, default_value = "11000")]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
pub struct CommandScenariosGenerate {}

impl Command {
    pub fn run(self) -> Result<(), crate::CommandError> {
        let rt = setup();
        let _rt_guard = rt.enter();

        match self {
            Self::Server(args) => {
                server(args.port);
                Ok(())
            }
            Self::ScenariosGenerate(_) => {
                #[cfg(feature = "scenario-generators")]
                {
                    for scenario in openmina_node_testing::scenarios::Scenarios::iter() {
                        rt.block_on(async {
                            scenario.run_and_save_from_scratch(Default::default()).await;
                        });
                    }
                    Ok(())
                }
                #[cfg(not(feature = "scenario-generators"))]
                Err("binary not compiled with `scenario-generators` feature"
                    .to_owned()
                    .into())
            }
        }
    }
}

pub fn main() {
    match OpenminaTestingCli::parse().command.run() {
        Ok(_) => {}
        Err(err) => exit_with_error(err),
    }
}

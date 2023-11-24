use clap::Parser;

use openmina_node_testing::scenarios::Scenarios;
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

    #[cfg(feature = "scenario-generators")]
    ScenariosGenerate(CommandScenariosGenerate),
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
}

impl Command {
    pub fn run(self) -> Result<(), crate::CommandError> {
        let rt = setup();
        let _rt_guard = rt.enter();

        match self {
            Self::Server(args) => {
                server(args.port);
                Ok(())
            }
            #[cfg(feature = "scenario-generators")]
            Self::ScenariosGenerate(cmd) => {
                if let Some(name) = cmd.name {
                    if let Some(scenario) = Scenarios::iter()
                        .into_iter()
                        .find(|s| <&'static str>::from(s) == name)
                    {
                        rt.block_on(scenario.run_and_save_from_scratch(Default::default()));
                    } else {
                        panic!("no such scenario: \"{name}\"");
                    }
                } else {
                    for scenario in Scenarios::iter() {
                        rt.block_on(scenario.run_and_save_from_scratch(Default::default()));
                    }
                }
                Ok(())
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

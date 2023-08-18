pub mod replay_state_with_input_actions;
pub use replay_state_with_input_actions::ReplayStateWithInputActions;

#[derive(Debug, clap::Args)]
pub struct Replay {
    #[command(subcommand)]
    pub command: ReplayCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum ReplayCommand {
    StateWithInputActions(ReplayStateWithInputActions),
}

impl Replay {
    pub fn run(self) -> Result<(), crate::CommandError> {
        match self.command {
            ReplayCommand::StateWithInputActions(v) => v.run(),
        }
    }
}

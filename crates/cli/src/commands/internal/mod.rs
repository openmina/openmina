pub mod graphql;

#[derive(Debug, clap::Args)]
pub struct Internal {
    #[command(subcommand)]
    pub command: InternalCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum InternalCommand {
    /// GraphQL endpoint introspection and management.
    Graphql(graphql::Graphql),
}

impl Internal {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            InternalCommand::Graphql(v) => v.run(),
        }
    }
}

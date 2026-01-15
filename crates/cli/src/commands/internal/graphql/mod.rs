pub mod inspect;
pub mod list;
pub mod run;

#[derive(Debug, clap::Args)]
pub struct Graphql {
    #[command(subcommand)]
    pub command: GraphqlCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum GraphqlCommand {
    /// List all available GraphQL endpoints.
    List(list::List),
    /// Inspect a GraphQL endpoint and display its schema.
    Inspect(inspect::Inspect),
    /// Execute a GraphQL query against the node.
    Run(run::Run),
}

impl Graphql {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            GraphqlCommand::List(v) => v.run(),
            GraphqlCommand::Inspect(v) => v.run(),
            GraphqlCommand::Run(v) => v.run(),
        }
    }
}

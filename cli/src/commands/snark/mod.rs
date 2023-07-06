pub mod precalculate_block_verifier_index_and_srs;
pub use precalculate_block_verifier_index_and_srs::PrecalculateBlockVerifierIndexAndSrs;

#[derive(Debug, clap::Args)]
pub struct Snark {
    #[command(subcommand)]
    pub command: SnarkCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum SnarkCommand {
    PrecalculateBlockVerifierIndexAndSrs(PrecalculateBlockVerifierIndexAndSrs),
}

impl Snark {
    pub fn run(self) -> Result<(), crate::CommandError> {
        match self.command {
            SnarkCommand::PrecalculateBlockVerifierIndexAndSrs(v) => v.run(),
        }
    }
}

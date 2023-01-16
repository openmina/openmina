pub mod precalculate_block_verifier_index_and_srs;
pub use precalculate_block_verifier_index_and_srs::PrecalculateBlockVerifierIndexAndSrs;

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "openmina", about = "Openmina Cli")]
pub enum Snark {
    PrecalculateBlockVerifierIndexAndSrs(PrecalculateBlockVerifierIndexAndSrs),
}

impl Snark {
    pub fn run(self) -> Result<(), crate::CommandError> {
        match self {
            Self::PrecalculateBlockVerifierIndexAndSrs(v) => v.run(),
        }
    }
}

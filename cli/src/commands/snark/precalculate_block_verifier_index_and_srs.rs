// use std::fs;
use std::path::PathBuf;

// use node::snark::{
//     get_srs, srs_to_bytes, verifier_index_to_bytes, VerifierKind,
// };
// use sha2::{Digest, Sha256};

#[derive(Debug, clap::Args)]
/// Precalculate Block Verifier Index and SRS, to be quickly loaded ready to be used for block verification
pub struct PrecalculateBlockVerifierIndexAndSrs {
    #[arg(default_value = ".")]
    pub out: PathBuf,
}

impl PrecalculateBlockVerifierIndexAndSrs {
    pub fn run(self) -> anyhow::Result<()> {
        unimplemented!()
        // let verifier_index =
        //     verifier_index_to_bytes(&get_verifier_index(VerifierKind::Blockchain))?;
        // let mut hasher = Sha256::new();
        // hasher.update(&verifier_index);
        // let index_hash = hex::encode(hasher.finalize());

        // let srs = {
        //     let srs = get_srs();
        //     srs_to_bytes(&srs)
        // };
        // let mut hasher = Sha256::new();
        // hasher.update(&srs);
        // let srs_hash = hex::encode(hasher.finalize());

        // let index_path = self.out.with_file_name("block_verifier_index.bin");
        // let srs_path = self.out.with_file_name("block_verifier_srs.bin");

        // fs::write(&index_path, verifier_index)?;
        // fs::write(&srs_path, srs)?;

        // eprintln!(
        //     "Precalculated Verifier Index: {:?}",
        //     fs::canonicalize(&index_path)?
        // );
        // eprintln!(
        //     "Precalculated Verifier SRS: {:?}",
        //     fs::canonicalize(&srs_path)?
        // );
        // eprintln!("Sha256 hashes represented as hex:");
        // println!("{}", index_hash);
        // println!("{}", srs_hash);

        // Ok(())
    }
}

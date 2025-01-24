use std::path::Path;

use mina_hasher::Fp;
use mina_p2p_messages::{binprot, v2};
use mina_tree::*;
use proofs::{block::BlockParams, constants::StepBlockProof, generate_block_proof, provers::{devnet_circuit_directory, BlockProver}, witness::Witness, wrap::WrapProof};
use rayon::ThreadPoolBuilder;

fn read_binprot<T, R>(mut r: R) -> T
where
    T: binprot::BinProtRead,
    R: std::io::Read,
{
    use std::io::Read;

    let mut len_buf = [0; std::mem::size_of::<u64>()];
    r.read_exact(&mut len_buf).unwrap();
    let len = u64::from_le_bytes(len_buf);

    let mut buf = Vec::with_capacity(len as usize);
    let mut r = r.take(len);
    r.read_to_end(&mut buf).unwrap();

    let mut read = buf.as_slice();
    T::binprot_read(&mut read).unwrap()
}

fn test_block_proof() {
    let Ok(data) = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(devnet_circuit_directory())
            .join("tests")
            .join("block_input-2483246-0.bin"),
    ) else {
        eprintln!("request not found");
        // panic_in_ci();
        return;
    };

    let blockchain_input: v2::ProverExtendBlockchainInputStableV2 =
        read_binprot(&mut data.as_slice());

    let BlockProver {
        block_step_prover,
        block_wrap_prover,
        tx_wrap_prover,
    } = BlockProver::make(None, None);

    dbg!(std::process::id());

    for i in 0..2 {

        if i == 1 {
            use std::io;
            use std::io::prelude::*;

            let stdin = io::stdin();
            println!("start ?");
            for line in stdin.lock().lines() {
                break; // break when user types a line
            }
        }

        let now = std::time::Instant::now();

        // let mut witnesses: Witness<Fp> = Witness::new::<StepBlockProof>();
        // witnesses.ocaml_aux = read_witnesses("block_fps.txt").unwrap();

        let WrapProof { proof, .. } = generate_block_proof(
            BlockParams {
                input: &blockchain_input,
                block_step_prover: &block_step_prover,
                block_wrap_prover: &block_wrap_prover,
                tx_wrap_prover: &tx_wrap_prover,
                only_verify_constraints: false,
                expected_step_proof: None,
                ocaml_wrap_witness: None,
                // expected_step_proof: Some(
                //     "a82a10e5c276dd6dc251241dcbad005201034ffff5752516a179f317dfe385f5",
                // ),
                // ocaml_wrap_witness: Some(read_witnesses("block_fqs.txt").unwrap()),
            },
            // &mut witnesses,
        )
            .unwrap();

        eprintln!("time: {:?}", now.elapsed());

        let proof_json = serde_json::to_vec(&proof.proof).unwrap();
        // let _sum = dbg!(sha256_sum(&proof_json));
    }
}

fn main() {

    let pool = ThreadPoolBuilder::new()
        .num_threads(1)
        .use_current_thread()
        .build_global()
        .unwrap();

    test_block_proof();
    // for naccounts in [1_000, 10_000, 120_000] {
    //     println!("{:?} accounts wasmer", naccounts);

    //     let now = redux::Instant::now();

    //     let mut db = Database::<V2>::create(20);

    //     let accounts = (0..naccounts).map(|_| Account::rand()).collect::<Vec<_>>();

    //     for (index, mut account) in accounts.into_iter().enumerate() {
    //         account.token_id = TokenId::from(index as u64);
    //         let id = account.id();
    //         db.get_or_create_account(id, account).unwrap();
    //     }

    //     println!("generate random accounts {:?}", now.elapsed());
    //     let now = redux::Instant::now();

    //     assert_eq!(db.num_accounts(), naccounts as usize);

    //     db.merkle_root();

    //     println!("compute merkle root {:?}", now.elapsed());
    // }
}

use binprot::BinProtRead;
use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaBlockHeaderStableV2;
use snark::{accumulator_check, get_srs, get_verifier_index, transition_chain, verify};

fn get_delta_block_chain_proof(header: &MinaBlockHeaderStableV2) -> (Fp, Vec<Fp>) {
    let delta_block_chain_proof = &header.delta_block_chain_proof;

    let init_state_hash: Fp = delta_block_chain_proof.0.to_field();
    let merkle_list: Vec<Fp> = delta_block_chain_proof
        .1
        .iter()
        .map(|f| f.to_field())
        .collect();

    (init_state_hash, merkle_list)
}

fn main() {
    assert!(std::env::args().len() > 1);

    let verifier_index = get_verifier_index();
    let srs = get_srs();

    for arg in std::env::args().skip(1) {
        let file = std::fs::read(&arg).unwrap();

        let header = if arg.ends_with(".json") {
            serde_json::from_slice::<MinaBlockHeaderStableV2>(&file).unwrap()
        } else {
            MinaBlockHeaderStableV2::binprot_read(&mut file.as_slice()).unwrap()
        };

        let target_hash: Fp = header.protocol_state.previous_state_hash.to_field();
        let transition_chain =
            transition_chain::verify(target_hash, get_delta_block_chain_proof(&header));
        assert!(transition_chain.is_some());

        let accum_check = accumulator_check(&srs, &header.protocol_state_proof.0);
        let verified = verify(header, &verifier_index);

        assert!(accum_check && verified);
        println!("{} valid", arg);
    }
}

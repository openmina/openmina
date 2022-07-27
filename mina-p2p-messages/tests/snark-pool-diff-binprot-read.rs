use mina_p2p_messages::p2p::NetworkPoolSnarkPoolDiffVersionedStable;
mod utils;

#[test]
#[ignore = "still failing"]
fn deserialize() {
    let encoded = utils::read("snark-pool-diff/1.bin").unwrap();
    let mut ptr = encoded.as_slice();
    let snark_pool_diff: NetworkPoolSnarkPoolDiffVersionedStable = binprot::BinProtRead::binprot_read(&mut ptr).unwrap();
    let json = serde_json::to_string_pretty(&snark_pool_diff).unwrap();
    eprintln!("{json}");
}

use mina_p2p_messages::p2p::NetworkPoolTransactionPoolDiffVersionedStable;

mod utils;

#[test]
fn deserialize() {
    let encoded = utils::read("tx-pool-diff/1.bin").unwrap();
    let mut ptr = encoded.as_slice();
    let tx_pool_diff: NetworkPoolTransactionPoolDiffVersionedStable = binprot::BinProtRead::binprot_read(&mut ptr).unwrap();
    let json = serde_json::to_string_pretty(&tx_pool_diff).unwrap();
    eprintln!("{json}");
}

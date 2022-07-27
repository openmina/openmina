use mina_p2p_messages::p2p::MinaBlockExternalTransitionRawVersionedStable;

mod utils;

#[test]
fn deserialize() {
    let encoded = utils::read("external-transition/1.bin").unwrap();
    let mut ptr = encoded.as_slice();
    let external_transition: MinaBlockExternalTransitionRawVersionedStable = binprot::BinProtRead::binprot_read(&mut ptr).unwrap();
    let json = serde_json::to_string_pretty(&external_transition).unwrap();
    eprintln!("{json}");
}

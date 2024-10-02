#![allow(unexpected_cfgs)]
#![cfg(all(test, fuzzing))]

fn try_decode<T>(mut buf: &[u8]) -> bool
where
    T: binprot::BinProtRead,
{
    match T::binprot_read(&mut buf) {
        Ok(_) => {}
        Err(_) => {}
    }
    true
}

fn fuzz<F>(f: F)
where
    F: Fn(&[u8]) -> bool + 'static,
{
    let result = fuzzcheck::fuzz_test(f)
        .default_options()
        .stop_after_first_test_failure(true)
        .launch();
    assert!(!result.found_test_failure);
}

#[test]
fn gossip_message() {
    fuzz(try_decode::<mina_p2p_messages::GossipNetMessageV1>);
}

#[test]
fn external_transition() {
    fuzz(try_decode::<mina_p2p_messages::p2p::MinaBlockExternalTransitionRawVersionedStable>);
}

#[test]
fn snark_pool_diff() {
    fuzz(try_decode::<mina_p2p_messages::p2p::NetworkPoolSnarkPoolDiffVersionedStable>);
}

#[test]
fn tx_pool_diff() {
    fuzz(try_decode::<mina_p2p_messages::p2p::NetworkPoolTransactionPoolDiffVersionedStable>);
}

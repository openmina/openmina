use mina_p2p_messages::p2p::{
    MinaBlockExternalTransitionRawVersionedStable, NetworkPoolSnarkPoolDiffVersionedStable,
    NetworkPoolTransactionPoolDiffVersionedStable,
};

mod utils;

#[test]
fn external_transition() {
    utils::for_all("external-transition", |encoded| {
        utils::assert_binprot_read::<MinaBlockExternalTransitionRawVersionedStable>(&encoded)
    })
    .unwrap();
}

#[test]
fn snark_pool_diff() {
    utils::for_all("snark-pool-diff", |encoded| {
        utils::assert_binprot_read::<NetworkPoolSnarkPoolDiffVersionedStable>(&encoded)
    })
    .unwrap();
}

#[test]
fn tx_pool_diff() {
    utils::for_all("tx-pool-diff", |encoded| {
        utils::assert_binprot_read::<NetworkPoolTransactionPoolDiffVersionedStable>(&encoded)
    })
    .unwrap();
}

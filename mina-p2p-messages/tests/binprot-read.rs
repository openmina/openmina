use mina_p2p_messages::v1::{
    MinaBlockExternalTransitionRawVersionedStableV1Binable,
    NetworkPoolSnarkPoolDiffVersionedStableV1Binable,
    NetworkPoolTransactionPoolDiffVersionedStableV1Binable,
};

mod utils;

#[test]
fn external_transition() {
    utils::for_all("external-transition", |encoded| {
        utils::assert_binprot_read::<MinaBlockExternalTransitionRawVersionedStableV1Binable>(
            &encoded,
        )
    })
    .unwrap();
}

#[test]
fn snark_pool_diff() {
    utils::for_all("snark-pool-diff", |encoded| {
        utils::assert_binprot_read::<NetworkPoolSnarkPoolDiffVersionedStableV1Binable>(&encoded)
    })
    .unwrap();
}

#[test]
fn tx_pool_diff() {
    utils::for_all("tx-pool-diff", |encoded| {
        utils::assert_binprot_read::<NetworkPoolTransactionPoolDiffVersionedStableV1Binable>(
            &encoded,
        )
    })
    .unwrap();
}

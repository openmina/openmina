use mina_p2p_messages::v1::{
    MinaBlockExternalTransitionRawVersionedStableV1Binable,
    NetworkPoolSnarkPoolDiffVersionedStableV1Binable,
    NetworkPoolTransactionPoolDiffVersionedStableV1Binable,
};

mod utils;

#[test]
fn external_transition_v1() {
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

#[test]
fn gossip_v2() {
    utils::for_all("v2/gossip", |encoded| {
        use mina_p2p_messages::gossip::GossipNetMessageV2;
        utils::assert_stream_read_and::<GossipNetMessageV2, _>(&encoded, |_| {})
    })
    .unwrap();
}

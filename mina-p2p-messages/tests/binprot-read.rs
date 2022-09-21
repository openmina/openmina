use mina_p2p_messages::v1::{
    MinaBlockExternalTransitionRawVersionedStableV1Versioned,
    NetworkPoolSnarkPoolDiffVersionedStableV1Versioned,
    NetworkPoolTransactionPoolDiffVersionedStableV1Versioned,
};

mod utils;

#[test]
fn external_transition_v1() {
    utils::for_all("external-transition", |encoded| {
        utils::assert_binprot_read::<MinaBlockExternalTransitionRawVersionedStableV1Versioned>(
            &encoded,
        )
    })
    .unwrap();
}

#[test]
fn snark_pool_diff() {
    utils::for_all("snark-pool-diff", |encoded| {
        utils::assert_binprot_read::<NetworkPoolSnarkPoolDiffVersionedStableV1Versioned>(&encoded)
    })
    .unwrap();
}

#[test]
fn tx_pool_diff() {
    utils::for_all("tx-pool-diff", |encoded| {
        utils::assert_binprot_read::<NetworkPoolTransactionPoolDiffVersionedStableV1Versioned>(
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

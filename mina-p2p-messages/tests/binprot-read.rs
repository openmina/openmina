use binprot::BinProtRead;
use mina_p2p_messages::{v1::{
    MinaBlockExternalTransitionRawVersionedStableV1Versioned,
    NetworkPoolSnarkPoolDiffVersionedStableV1Versioned,
    NetworkPoolTransactionPoolDiffVersionedStableV1Versioned,
}, gossip::GossipNetMessageV2};

mod utils;

#[test]
fn external_transition_v1() {
    utils::for_all("v1/gossip/external-transition", |_, encoded| {
        utils::assert_binprot_read::<MinaBlockExternalTransitionRawVersionedStableV1Versioned>(
            &encoded,
        )
    })
    .unwrap();
}

#[test]
fn snark_pool_diff() {
    utils::for_all("v1/gossip/snark-pool-diff", |_, encoded| {
        utils::assert_binprot_read::<NetworkPoolSnarkPoolDiffVersionedStableV1Versioned>(&encoded)
    })
    .unwrap();
}

#[test]
fn tx_pool_diff() {
    utils::for_all("v1/gossip/tx-pool-diff", |_, encoded| {
        utils::assert_binprot_read::<NetworkPoolTransactionPoolDiffVersionedStableV1Versioned>(
            &encoded,
        )
    })
    .unwrap();
}

#[test]
fn gossip_v2() {
    utils::for_all("v2/gossip", |_, encoded| {
        use mina_p2p_messages::gossip::GossipNetMessageV2;
        utils::assert_binprot_read::<GossipNetMessageV2>(&encoded)
    })
    .unwrap();
}


#[test]
fn g() {
    let binprot = include_bytes!("files/test_data_2.bin");
    GossipNetMessageV2::binprot_read(&mut &binprot[..]).unwrap();
    let binprot = include_bytes!("files/test_data_3.bin");
    GossipNetMessageV2::binprot_read(&mut &binprot[..]).unwrap();
}

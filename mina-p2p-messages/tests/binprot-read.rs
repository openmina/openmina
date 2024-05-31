// use mina_p2p_messages::v1::{
//     MinaBlockExternalTransitionRawVersionedStableV1Versioned,
//     NetworkPoolSnarkPoolDiffVersionedStableV1Versioned,
//     NetworkPoolTransactionPoolDiffVersionedStableV1Versioned,
// };

mod utils;

// TODO: v1 got removed, what should replace this?
// #[test]
// fn external_transition_v1() {
//     utils::for_all("v1/gossip/external-transition", |_, encoded| {
//         utils::assert_binprot_read::<MinaBlockExternalTransitionRawVersionedStableV1Versioned>(
//             encoded,
//         )
//     })
//     .unwrap();
// }
//
// #[test]
// fn snark_pool_diff() {
//     utils::for_all("v1/gossip/snark-pool-diff", |_, encoded| {
//         utils::assert_binprot_read::<NetworkPoolSnarkPoolDiffVersionedStableV1Versioned>(encoded)
//     })
//     .unwrap();
// }
//
// #[test]
// fn tx_pool_diff() {
//     utils::for_all("v1/gossip/tx-pool-diff", |_, encoded| {
//         utils::assert_binprot_read::<NetworkPoolTransactionPoolDiffVersionedStableV1Versioned>(
//             encoded,
//         )
//     })
//     .unwrap();
// }

#[ignore = "need to fix bin files in `v2/gossip`"]
#[test]
fn gossip_v2() {
    utils::for_all("v2/gossip", |_, encoded| {
        use mina_p2p_messages::gossip::GossipNetMessageV2;
        utils::assert_binprot_read::<GossipNetMessageV2>(encoded)
    })
    .unwrap();
}

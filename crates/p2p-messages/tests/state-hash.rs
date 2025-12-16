mod utils;

// TODO: v1 got removed, what should replace this?
// #[cfg(feature = "hashing")]
// mod tests {
//     use binprot::BinProtRead;
//     use mina_p2p_messages::v1::MinaBlockExternalTransitionRawVersionedStableV1Versioned;
//
//     #[test]
//     fn state_hash() {
//         let data: [(&[u8], &str); 2] = [
//             (
//                 include_bytes!("files/v1/gossip/external-transition/1.bin"),
//                 "3NLUiFWnakJxjUhFoNw1PPSRuRgJbhmfHqSLweEWrxcPpo8ikVTb",
//             ),
//             (
//                 include_bytes!("files/v1/gossip/external-transition/2.bin"),
//                 "3NL7hhVbBfxhWfyUscv6LrqfvxgHGfQVPMi8ynDMZyyDN2GhqDbc",
//             ),
//         ];
//         for (mut encoded, expected_hash) in data {
//             let external_transition =
//                 MinaBlockExternalTransitionRawVersionedStableV1Versioned::binprot_read(
//                     &mut encoded,
//                 )
//                 .unwrap();
//             let json =
//                 serde_json::to_string_pretty(&external_transition.inner().protocol_state).unwrap();
//             eprintln!("{json}");
//             let mut hasher = mina_hasher::create_legacy(());
//             let hash = external_transition.inner().protocol_state.hash(&mut hasher);
//
//             assert_eq!(&hash.to_string(), expected_hash);
//         }
//     }
// }

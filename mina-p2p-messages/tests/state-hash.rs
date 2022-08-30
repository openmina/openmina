#[cfg(feature = "hashing")]
mod utils;

#[cfg(feature = "hashing")]
mod tests {
    use super::*;
    use binprot::BinProtRead;
    use mina_p2p_messages::v1::MinaBlockExternalTransitionRawVersionedStableV1Versioned;

    #[test]
    fn state_hash() {
        utils::for_all("v1/gossip/external-transition", |file_path, mut encoded| {
            let file_path = file_path.to_string_lossy();
            let external_transition = MinaBlockExternalTransitionRawVersionedStableV1Versioned::binprot_read(&mut encoded).unwrap();
            let json = serde_json::to_string_pretty(&external_transition.inner().protocol_state).unwrap();
            eprintln!("{json}");
            let mut hasher = mina_hasher::create_legacy(());
            let hash = external_transition.inner().protocol_state.hash(&mut hasher);

            if file_path.ends_with("v1/gossip/external-transition/1.bin") {
                assert_eq!(
                    hash.to_string(),
                    "3NLUiFWnakJxjUhFoNw1PPSRuRgJbhmfHqSLweEWrxcPpo8ikVTb".to_owned()
                );
            } else if file_path.ends_with("v1/gossip/external-transition/2.bin") {
                assert_eq!(
                    hash.to_string(),
                    "3NL7hhVbBfxhWfyUscv6LrqfvxgHGfQVPMi8ynDMZyyDN2GhqDbc".to_owned()
                );
            } else {
                panic!("Unknown test file. Is it new? Add it to `tests/state-hash.rs`");
            }
        })
        .unwrap();
    }
}

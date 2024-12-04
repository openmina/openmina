#[cfg(feature = "p2p-webrtc")]
use openmina_node_testing::{scenarios::Scenarios, setup};

#[cfg(feature = "p2p-webrtc")]
#[test]
fn node_libp2p_with_rust_to_rust_webrtc_all_scenarios() {
    let rt = setup();

    for scenario in Scenarios::iter() {
        eprintln!("running scenario: {}", scenario.to_str());
        let mut config = scenario.default_cluster_config().unwrap();
        config.set_all_rust_to_rust_use_webrtc();
        rt.block_on(async move {
            scenario.run_only_from_scratch(config).await;
        });
    }
}

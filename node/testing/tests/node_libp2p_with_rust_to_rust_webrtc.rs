#[cfg(feature = "p2p-webrtc")]
use openmina_node_testing::{cluster::ClusterConfig, scenarios::Scenarios, setup};

#[cfg(feature = "p2p-webrtc")]
#[test]
fn node_libp2p_with_rust_to_rust_webrtc_all_scenarios() {
    let rt = setup();
    let config = ClusterConfig::new(None)
        .unwrap()
        .set_all_rust_to_rust_use_webrtc();

    for scenario in Scenarios::iter() {
        eprintln!("running scenario: {}", scenario.to_str());
        rt.block_on(async {
            scenario.run_only_from_scratch(config.clone()).await;
        });
    }
}

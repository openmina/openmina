#![cfg(feature = "scenario-generators")]

use openmina_node_testing::{cluster::ClusterConfig, scenarios::Scenarios, setup};

#[test]
fn node_libp2p_only_all_scenarios() {
    let rt = setup();
    let config = ClusterConfig::default();

    for scenario in Scenarios::iter() {
        eprintln!("running scenario: {}", scenario.to_str());
        rt.block_on(async {
            scenario.run_only_from_scratch(config.clone()).await;
        });
    }
}

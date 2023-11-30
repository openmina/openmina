#[macro_export]
macro_rules! scenario_doc {
    ($doc:expr) => {
        if std::env::var("SCENARIO_INFO").map_or(false, |s| !s.is_empty()) {
            println!("{}", $doc);
            return;
        }
    }
}

#[macro_export]
macro_rules! scenario_test {
    ($name:ident, $scenario:ty, $scenario_instance:expr) => {
        #[tokio::test]
        async fn $name() {
            use openmina_node_testing::{
                cluster::{Cluster, ClusterConfig},
                scenarios::ClusterRunner,
            };

            if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY") {
                let _ = std::fs::write(&summary, format!("### `{}`\n\n{}\n\n", stringify!($name), <$scenario as documented::Documented>::DOCS));
                std::panic::set_hook(Box::new(move |_panic_info| {
                    let _ = std::fs::write(&summary, "**FAILED** :red_circle:");
                }));
            }

            openmina_node_testing::setup_without_rt();
            let config = ClusterConfig::default();
            let mut cluster = Cluster::new(config);
            let runner = ClusterRunner::new(&mut cluster, |_| {});
            let scenario = $scenario_instance;
            scenario.run(runner).await;

            if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY") {
                let _ = std::fs::write(summary, "**PASSED** :white_check_mark:");
            }
        }
    };
}

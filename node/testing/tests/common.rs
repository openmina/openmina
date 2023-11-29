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
            scenario_doc!(<$scenario>::DOCS);
            openmina_node_testing::setup_without_rt();
            let config = ClusterConfig::default();
            let mut cluster = Cluster::new(config);
            let runner = openmina_node_testing::scenarios::ClusterRunner::new(&mut cluster, |_| {});
            let scenario = $scenario_instance;
            scenario.run(runner).await
        }
    };
}

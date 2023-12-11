#[macro_export]
macro_rules! scenario_doc {
    ($doc:expr) => {
        if std::env::var("SCENARIO_INFO").map_or(false, |s| !s.is_empty()) {
            println!("{}", $doc);
            return;
        }
    };
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
            use std::io::Write;

            if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY") {
                let _ = std::fs::File::options()
                    .append(true)
                    .open(&summary)
                    .and_then(|mut f| {
                        writeln!(
                            f,
                            "### `{}`\n\n{}\n",
                            stringify!($name),
                            <$scenario as documented::Documented>::DOCS
                        )
                    });
                let prev_panic_hook = std::panic::take_hook();
                std::panic::set_hook(Box::new(move |panic_info| {
                    let _ = std::fs::File::options()
                        .append(true)
                        .open(&summary)
                        .and_then(|mut f| writeln!(f, "**FAILED** :red_circle:"));
                    if let Some((file, line)) = panic_info.location().map(|l| (l.file(), l.line()))
                    {
                        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
                            eprintln!("\n::error file={file},line={line}::{message}");
                        } else {
                            eprintln!("\n::error file={file},line={line}::panic without a message");
                        }
                    }
                    prev_panic_hook(panic_info);
                }));
            }

            let config = ClusterConfig::default();
            let mut cluster = Cluster::new(config);
            let runner = ClusterRunner::new(&mut cluster, |_| {});
            let scenario = $scenario_instance;
            scenario.run(runner).await;

            if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY") {
                let _ = std::fs::File::options()
                    .append(true)
                    .open(&summary)
                    .and_then(|mut f| writeln!(f, "**PASSED** :white_check_mark:"));
            }
        }
    };
}

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
    ($(#[$meta:meta])? $name:ident, $scenario:ty, $scenario_instance:expr) => {
        scenario_test!($(#[$meta])? $name, $scenario, $scenario_instance, false);
    };
    ($(#[$meta:meta])? $name:ident, $scenario:ty, $scenario_instance:expr, $can_test_webrtc:expr) => {
        #[tokio::test]
        $(#[$meta])?
        async fn $name() {
            use openmina_node_testing::{
                cluster::{Cluster, ClusterConfig},
                scenarios::ClusterRunner,
                setup_without_rt, wait_for_other_tests,
            };
            use std::io::Write;

            setup_without_rt();
            let w = wait_for_other_tests().await;

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
                            println!("\n::error file={file},line={line}::{message}");
                        } else {
                            println!("\n::error file={file},line={line}::panic without a message");
                        }
                    }
                    prev_panic_hook(panic_info);
                }));
            }

            #[allow(unused_mut)]
            let mut config = ClusterConfig::new(None).unwrap();
            #[cfg(feature = "p2p-webrtc")]
            if $can_test_webrtc {
                eprintln!("All rust to rust connections will be over webrtc transport");
                config = config.set_all_rust_to_rust_use_webrtc();
            }
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
            w.release();
        }
    };
}

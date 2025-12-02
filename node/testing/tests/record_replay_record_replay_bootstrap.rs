use mina_node_testing::scenarios::record_replay::bootstrap::RecordReplayBootstrap;

mod common;

// To run locally:
// ```bash
// export MINA_DISCOVERY_FILTER_ADDR=false
// export KEEP_CONNECTION_WITH_UNKNOWN_STREAM=true
// export REPLAYER_MULTIADDR=/dns4/mina-rust-ci-1-libp2p.gcp.o1test.net/tcp/8302/p2p/12D3KooWQi9rSWT2kmEavbEc5eP13nG1FRStMiERKZB3wPJSkNrE
// export BPF_ALIAS=/coda/0.0.1/29936104443aaf264a7f0192ac64b1c7173198c1ed404c1bcff5e562e05eb7f6-0.0.0.0
// cargo test -r --package mina-node-testing --test record_replay_record_replay_bootstrap -- record_replay_bootstrap --exact --nocapture
// ```
scenario_test!(
    record_replay_bootstrap,
    RecordReplayBootstrap,
    RecordReplayBootstrap,
    true
);

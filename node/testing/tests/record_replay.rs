use openmina_node_testing::scenarios::record_replay::{
    block_production::RecordReplayBlockProduction, bootstrap::RecordReplayBootstrap,
};

mod common;

scenario_test!(
    record_replay_bootstrap,
    RecordReplayBootstrap,
    RecordReplayBootstrap
);

scenario_test!(
    record_replay_block_production,
    RecordReplayBlockProduction,
    RecordReplayBlockProduction
);

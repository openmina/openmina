use mina_node_testing::scenarios::record_replay::block_production::RecordReplayBlockProduction;

mod common;

scenario_test!(
    record_replay_block_production,
    RecordReplayBlockProduction,
    RecordReplayBlockProduction,
    true
);

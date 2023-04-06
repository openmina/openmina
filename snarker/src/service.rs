use std::collections::BTreeMap;

pub use redux::TimeService;
use redux::{ActionMeta, ActionWithMeta};
use serde::{Deserialize, Serialize};

pub use crate::event_source::EventSourceService;
pub use crate::p2p::channels::P2pChannelsService;
pub use crate::p2p::connection::P2pConnectionService;
pub use crate::p2p::disconnection::P2pDisconnectionService;
pub use crate::rpc::RpcService;
use crate::ActionKind;

pub trait Service:
    TimeService
    + EventSourceService
    + P2pConnectionService
    + P2pDisconnectionService
    + P2pChannelsService
    + RpcService
{
    fn stats(&mut self) -> Option<&mut Stats>;
}

pub type ActionKindWithMeta = ActionWithMeta<ActionKind>;

pub struct Stats {
    last_action: ActionKindWithMeta,
    action_stats: ActionStats,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            last_action: ActionMeta::ZERO.with_action(ActionKind::None),
            action_stats: Default::default(),
        }
    }

    pub fn new_action(&mut self, kind: ActionKind, meta: ActionMeta) -> &mut Self {
        let action = meta.with_action(kind);
        self.action_stats.add(&action, &self.last_action);
        self.last_action = action;
        self
    }

    pub fn collect_action_stats_since_start(&self) -> BTreeMap<ActionKind, ActionStatsForRanges> {
        self.action_stats
            .since_start
            .iter()
            .enumerate()
            .map(|(i, v)| ((i as u16).try_into().unwrap(), v.clone()))
            .collect()
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Clone)]
pub struct ActionStats {
    /// Stats since the start of the node, indexed by `ActionKind`.
    since_start: Vec<ActionStatsForRanges>,
}

impl ActionStats {
    pub fn add(&mut self, action: &ActionKindWithMeta, prev_action: &ActionKindWithMeta) {
        if *prev_action.action() == ActionKind::None {
            return;
        }

        let kind_i = *prev_action.action() as usize;
        let duration = action.meta().time_as_nanos() - prev_action.meta().time_as_nanos();

        // TODO(binier): add constant len in ActionKind instead and use
        // that for constant vec length.
        let len = self.since_start.len();
        let need_len = kind_i + 1;
        if len < need_len {
            self.since_start.resize(need_len, Default::default());
        }
        self.since_start[kind_i].add(duration);
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ActionStatsForRange {
    /// Total number of times this action kind was executed.
    pub total_calls: u64,
    /// Sum of durations from this action till the next one in nanoseconds.
    pub total_duration: u64,
    /// Max duration.
    pub max_duration: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ActionStatsForRanges {
    pub under_1_us: ActionStatsForRange,
    pub under_10_us: ActionStatsForRange,
    pub under_50_us: ActionStatsForRange,
    pub under_100_us: ActionStatsForRange,
    pub under_500_us: ActionStatsForRange,
    pub under_1_ms: ActionStatsForRange,
    pub under_5_ms: ActionStatsForRange,
    pub above_5_ms: ActionStatsForRange,
}

impl ActionStatsForRanges {
    pub fn add(&mut self, duration: u64) {
        let stats = if duration <= 1_000 {
            &mut self.under_1_us
        } else if duration <= 10_000 {
            &mut self.under_10_us
        } else if duration <= 50_000 {
            &mut self.under_50_us
        } else if duration <= 100_000 {
            &mut self.under_100_us
        } else if duration <= 500_000 {
            &mut self.under_500_us
        } else if duration <= 1_000_000 {
            &mut self.under_1_ms
        } else if duration <= 5_000_000 {
            &mut self.under_5_ms
        } else {
            &mut self.above_5_ms
        };
        stats.total_calls += 1;
        stats.total_duration += duration;
        stats.max_duration = std::cmp::max(stats.max_duration, duration);
    }
}

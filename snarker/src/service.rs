use std::collections::{BTreeMap, VecDeque};

use mina_p2p_messages::v2::StateHash;
pub use redux::TimeService;
use redux::{ActionMeta, ActionWithMeta, Timestamp};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use shared::block::{Block, BlockWithHash};

pub use crate::event_source::EventSourceService;
pub use crate::p2p::channels::P2pChannelsService;
pub use crate::p2p::connection::P2pConnectionService;
pub use crate::p2p::disconnection::P2pDisconnectionService;
pub use crate::rpc::RpcService;
pub use crate::snark::block_verify::SnarkBlockVerifyService;
use crate::ActionKind;

pub trait Service:
    TimeService
    + EventSourceService
    + SnarkBlockVerifyService
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
        let mut action_stats_per_block = VecDeque::new();
        action_stats_per_block.push_back(ActionStatsForBlock {
            id: 0,
            time: Timestamp::ZERO,
            block_level: 1,
            // TODO(binier): use configured genesis hash.
            block_hash: "3NKeMoncuHab5ScarV5ViyF16cJPT4taWNSaTLS64Dp67wuXigPZ"
                .parse()
                .unwrap(),
            cpu_idle: 0,
            cpu_busy: 0,
            stats: Default::default(),
        });
        Self {
            last_action: ActionMeta::ZERO.with_action(ActionKind::None),
            action_stats: ActionStats {
                since_start: Default::default(),
                per_block: action_stats_per_block,
            },
        }
    }

    pub fn new_best_tip<T: AsRef<Block>>(
        &mut self,
        time: Timestamp,
        block: &BlockWithHash<T>,
    ) -> &mut Self {
        let level = block
            .block
            .as_ref()
            .header
            .protocol_state
            .body
            .consensus_state
            .blockchain_length
            .0
            .as_u32();
        self.action_stats
            .new_best_tip(time, level, block.hash.clone());
        self
    }

    pub fn new_action(&mut self, kind: ActionKind, meta: ActionMeta) -> &mut Self {
        let action = meta.with_action(kind);
        self.action_stats.add(&action, &self.last_action);
        self.last_action = action;
        self
    }

    pub fn collect_action_stats_since_start(&self) -> ActionStatsSnapshot {
        self.action_stats.since_start.clone()
    }

    pub fn collect_action_stats_for_block_with_id(
        &self,
        id: Option<u64>,
    ) -> Option<ActionStatsForBlock> {
        let blocks = &self.action_stats.per_block;
        let last = blocks.back()?;
        let id = match id {
            Some(id) => {
                if id == last.id {
                    return Some(last.clone());
                }
                id
            }
            None => return Some(last.clone()),
        };
        let i = id
            .checked_add(blocks.len() as u64)?
            .checked_sub(last.id + 1)?;
        blocks.get(i as usize).cloned()
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
    since_start: ActionStatsSnapshot,
    per_block: VecDeque<ActionStatsForBlock>,
}

impl ActionStats {
    pub fn new_best_tip(&mut self, time: Timestamp, level: u32, hash: StateHash) {
        while self.per_block.len() >= 20000 {
            self.per_block.pop_back();
        }
        let id = self.per_block.back().map_or(0, |v| v.id + 1);
        self.per_block.push_back(ActionStatsForBlock {
            id,
            time,
            block_level: level,
            block_hash: hash,
            cpu_idle: 0,
            cpu_busy: 0,
            stats: Default::default(),
        });
    }

    pub fn add(&mut self, action: &ActionKindWithMeta, prev_action: &ActionKindWithMeta) {
        self.since_start.add(action, prev_action);
        if let Some(stats) = self.per_block.back_mut() {
            stats.new_action(action, prev_action);
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActionStatsSnapshot(Vec<ActionStatsForRanges>);

impl ActionStatsSnapshot {
    pub fn add(&mut self, action: &ActionKindWithMeta, prev_action: &ActionKindWithMeta) {
        if *prev_action.action() == ActionKind::None {
            return;
        }

        let kind_i = *prev_action.action() as usize;
        let duration = action.meta().time_as_nanos() - prev_action.meta().time_as_nanos();

        // TODO(binier): add constant len in ActionKind instead and use
        // that for constant vec length.
        let len = self.0.len();
        let need_len = kind_i + 1;
        if len < need_len {
            self.0.resize(need_len, Default::default());
        }
        self.0[kind_i].add(duration);
    }
}

impl Serialize for ActionStatsSnapshot {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut m = s.serialize_map(Some(self.0.len()))?;
        self.0
            .iter()
            .enumerate()
            .skip(1) // skip `None` action
            .map(|(i, v)| (ActionKind::try_from(i as u16).unwrap(), v))
            .try_for_each(|(k, v)| m.serialize_entry(&k, v))?;
        m.end()
    }
}

impl<'de> Deserialize<'de> for ActionStatsSnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut m: BTreeMap<ActionKind, ActionStatsForRanges> =
            Deserialize::deserialize(deserializer)?;
        let list = (0..(ActionKind::COUNT as u16))
            .map(|i| {
                let kind = i.try_into().unwrap();
                m.remove(&kind).unwrap_or(ActionStatsForRanges::default())
            })
            .collect();
        Ok(Self(list))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionStatsForBlock {
    pub id: u64,
    pub time: Timestamp,
    pub block_level: u32,
    pub block_hash: StateHash,
    pub cpu_idle: u64,
    pub cpu_busy: u64,
    pub stats: ActionStatsSnapshot,
}

impl ActionStatsForBlock {
    fn new_action(&mut self, action: &ActionKindWithMeta, prev_action: &ActionKindWithMeta) {
        let duration = action.meta().time_as_nanos() - prev_action.meta().time_as_nanos();
        match prev_action.action() {
            ActionKind::None => {}
            ActionKind::EventSourceWaitForEvents => self.cpu_idle += duration,
            _ => self.cpu_busy += duration,
        }
        self.stats.add(action, prev_action);
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
    pub under_50_ms: ActionStatsForRange,
    pub above_50_ms: ActionStatsForRange,
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
        } else if duration <= 50_000_000 {
            &mut self.under_50_ms
        } else {
            &mut self.above_50_ms
        };
        stats.total_calls += 1;
        stats.total_duration += duration;
        stats.max_duration = std::cmp::max(stats.max_duration, duration);
    }
}

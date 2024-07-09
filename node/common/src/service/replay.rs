use std::collections::VecDeque;

use node::ActionKind;
use redux::ActionMeta;

pub struct ReplayerState {
    pub initial_monotonic: redux::Instant,
    pub initial_time: redux::Timestamp,
    pub expected_actions: VecDeque<(ActionKind, ActionMeta)>,
    pub replay_dynamic_effects_lib: String,
}

impl ReplayerState {
    pub fn next_monotonic_time(&self) -> redux::Instant {
        self.expected_actions
            .front()
            .map(|(_, meta)| meta.time())
            .map(|expected_time| {
                let time_passed = expected_time.checked_sub(self.initial_time).unwrap();
                self.initial_monotonic + time_passed
            })
            .unwrap_or(self.initial_monotonic)
    }
}

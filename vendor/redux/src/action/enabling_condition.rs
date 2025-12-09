#[allow(unused_variables)]
pub trait EnablingCondition<State> {
    /// Enabling condition for the Action.
    ///
    /// Checks if the given action is enabled for a given state and timestamp.
    fn is_enabled(&self, state: &State, time: crate::Timestamp) -> bool {
        true
    }
}

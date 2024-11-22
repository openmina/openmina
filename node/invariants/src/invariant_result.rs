use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InvariantResult {
    Ignored(InvariantIgnoreReason),
    /// Invariant check was triggered but as a result we didn't do any
    /// checks, instead internal state of invariant might have been updated.
    Updated,
    /// Invariant has been violated!
    Violation(String),
    /// Invariant check was done and it passed.
    Ok,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InvariantIgnoreReason {
    GlobalInvariantNotInTestingCluster,
}

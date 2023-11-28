use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct ClusterNodeId(usize);

impl ClusterNodeId {
    pub fn new_unchecked(i: usize) -> Self {
        Self(i)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for ClusterNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ClusterNodeId> for u64 {
    fn from(value: ClusterNodeId) -> Self {
        value.0 as u64
    }
}

impl From<ClusterNodeId> for usize {
    fn from(value: ClusterNodeId) -> Self {
        value.0
    }
}

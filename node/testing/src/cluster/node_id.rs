use std::{fmt, collections::BTreeSet};

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

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct ClusterOcamlNodeId(usize);

impl ClusterOcamlNodeId {
    pub fn new_unchecked(i: usize) -> Self {
        Self(i)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for ClusterOcamlNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ocaml_{}", self.0)
    }
}

impl From<ClusterOcamlNodeId> for u64 {
    fn from(value: ClusterOcamlNodeId) -> Self {
        value.0 as u64
    }
}

impl From<ClusterOcamlNodeId> for usize {
    fn from(value: ClusterOcamlNodeId) -> Self {
        value.0
    }
}

pub struct ClusterNodeIdGenerator {
    released_ids: Vec<ClusterNodeId>,
    used_ids: BTreeSet<ClusterNodeId>,
}

impl ClusterNodeIdGenerator {
    pub fn new() -> Self {
        Self {
            released_ids: Vec::new(),
            used_ids: BTreeSet::new(),
        }
    }

    pub fn new_id(&mut self) -> ClusterNodeId {
        if let Some(node_id) = self.released_ids.pop() {
            node_id
        } else {
            let node_id = ClusterNodeId::new_unchecked(self.used_ids.len());
            self.used_ids.insert(node_id);
            node_id
        }
    }

    pub fn release_id(&mut self, node_id: ClusterNodeId) {
        let removed = self.used_ids.remove(&node_id);

        if removed {
            self.released_ids.push(node_id)
        }
    }
}
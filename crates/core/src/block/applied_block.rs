use std::{ops::Deref, sync::Arc};

use super::ArcBlockWithHash;
use mina_p2p_messages::v2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppliedBlock {
    pub block: ArcBlockWithHash,
    pub just_emitted_a_proof: bool,
}

impl std::cmp::PartialEq for AppliedBlock {
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block
    }
}

impl Deref for AppliedBlock {
    type Target = ArcBlockWithHash;

    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

impl AppliedBlock {
    pub fn block_with_hash(&self) -> &ArcBlockWithHash {
        &self.block
    }

    pub fn block(&self) -> &Arc<v2::MinaBlockBlockStableV2> {
        &self.block_with_hash().block
    }
}

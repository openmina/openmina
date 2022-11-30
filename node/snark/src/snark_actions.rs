use serde::{Deserialize, Serialize};

use super::block_verify::SnarkBlockVerifyAction;

pub type SnarkActionWithMeta = redux::ActionWithMeta<SnarkAction>;
pub type SnarkActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkAction {
    BlockVerify(SnarkBlockVerifyAction),
}

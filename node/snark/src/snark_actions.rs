use serde::{Deserialize, Serialize};

use super::block_verify::SnarkBlockVerifyAction;
use super::work_verify::SnarkWorkVerifyAction;

pub type SnarkActionWithMeta = redux::ActionWithMeta<SnarkAction>;
pub type SnarkActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkAction {
    BlockVerify(SnarkBlockVerifyAction),
    WorkVerify(SnarkWorkVerifyAction),
}

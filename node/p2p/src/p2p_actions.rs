use serde::{Deserialize, Serialize};

use crate::pubsub::P2pPubsubAction;

use super::connection::P2pConnectionAction;

pub type P2pActionWithMeta = redux::ActionWithMeta<P2pAction>;
pub type P2pActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pAction {
    Connection(P2pConnectionAction),
    Pubsub(P2pPubsubAction),
}

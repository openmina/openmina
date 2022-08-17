use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Serialize, Deserialize};

pub mod p2p;
pub mod versioned;
pub mod bigint;
pub mod phantom;
pub mod char_;
pub mod string;

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub enum GossipNetMessage {
    NewState(p2p::MinaBlockExternalTransitionRawVersionedStable),
    SnarkPoolDiff(p2p::NetworkPoolSnarkPoolDiffVersionedStable),
    TransactionPoolDiff(p2p::NetworkPoolTransactionPoolDiffVersionedStable),
}

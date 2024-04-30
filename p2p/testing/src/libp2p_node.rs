use p2p::PeerId;

use crate::test_node::TestNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Libp2pNodeId(pub(super) usize);

pub struct Libp2pNode {}

impl TestNode for Libp2pNode {
    fn peer_id(&self) -> PeerId {
        todo!()
    }

    fn libp2p_port(&self) -> u16 {
        todo!()
    }
}

#[derive(Debug)]
pub struct Libp2pEvent;

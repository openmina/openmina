use crate::PeerId;

use super::{P2pRpcId, P2pRpcRequest};

pub trait P2pRpcService: redux::Service {
    /// Initate outgoing rpc request.
    fn outgoing_init(&mut self, peer_id: PeerId, id: P2pRpcId, req: P2pRpcRequest);
}

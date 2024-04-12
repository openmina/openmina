use redux::ActionWithMeta;

use crate::P2pNetworkKademliaRpcRequest;

use super::{P2pNetworkKadRequestAction, P2pNetworkKadRequestState};

impl P2pNetworkKadRequestState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKadRequestAction>,
    ) -> Result<(), String> {
        let (action, _meta) = action.split();
        use super::P2pNetworkKadRequestStatus as S;
        use P2pNetworkKadRequestAction as A;

        match action {
            A::New { .. } => {}
            A::PeerIsConnecting { .. } => self.status = S::WaitingForConnection,
            A::MuxReady { .. } => {}
            A::StreamIsCreating { stream_id, .. } => {
                self.status = S::WaitingForKadStream(*stream_id)
            }
            A::StreamReady { .. } => {
                let find_node = P2pNetworkKademliaRpcRequest::FindNode { key: self.key };
                let message = super::super::Message::from(&find_node);
                self.status = quick_protobuf::serialize_into_vec(&message).map_or_else(
                    |e| S::Error(format!("error serializing message: {e}")),
                    S::Request,
                );
            }
            A::RequestSent { .. } => self.status = S::WaitingForReply,
            A::ReplyReceived { data, .. } => self.status = S::Reply(data.clone()),
            A::Prune { .. } => return Err(String::from("should never happen")),
            A::Error { error, .. } => self.status = S::Error(error.clone()),
        }

        Ok(())
    }
}

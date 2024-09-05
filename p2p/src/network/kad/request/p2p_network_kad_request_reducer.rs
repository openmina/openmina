use redux::ActionWithMeta;

use crate::P2pNetworkKademliaRpcRequest;

use super::{P2pNetworkKadRequestAction, P2pNetworkKadRequestState};

impl P2pNetworkKadRequestState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKadRequestAction>,
    ) -> Result<(), String> {
        let (action, _meta) = action.split();

        match action {
            P2pNetworkKadRequestAction::New { .. } => {}
            P2pNetworkKadRequestAction::PeerIsConnecting { .. } => {
                self.status = super::P2pNetworkKadRequestStatus::WaitingForConnection
            }
            P2pNetworkKadRequestAction::MuxReady { .. } => {}
            P2pNetworkKadRequestAction::StreamIsCreating { stream_id, .. } => {
                self.status = super::P2pNetworkKadRequestStatus::WaitingForKadStream(*stream_id)
            }
            P2pNetworkKadRequestAction::StreamReady { .. } => {
                let find_node =
                    P2pNetworkKademliaRpcRequest::find_node(self.key).map_err(|e| e.to_string())?;
                let message = super::super::Message::from(&find_node);
                self.status = quick_protobuf::serialize_into_vec(&message).map_or_else(
                    |e| {
                        super::P2pNetworkKadRequestStatus::Error(format!(
                            "error serializing message: {e}"
                        ))
                    },
                    super::P2pNetworkKadRequestStatus::Request,
                );
            }
            P2pNetworkKadRequestAction::RequestSent { .. } => {
                self.status = super::P2pNetworkKadRequestStatus::WaitingForReply
            }
            P2pNetworkKadRequestAction::ReplyReceived { data, .. } => {
                self.status = super::P2pNetworkKadRequestStatus::Reply(data.clone())
            }
            P2pNetworkKadRequestAction::Prune { .. } => {
                return Err(String::from("should never happen"))
            }
            P2pNetworkKadRequestAction::Error { error, .. } => {
                self.status = super::P2pNetworkKadRequestStatus::Error(error.clone())
            }
        }

        Ok(())
    }
}

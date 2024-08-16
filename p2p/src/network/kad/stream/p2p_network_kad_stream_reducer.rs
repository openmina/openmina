use quick_protobuf::{serialize_into_vec, BytesReader};
use redux::ActionWithMeta;

use crate::{
    P2pLimits, P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest,
    P2pNetworkStreamProtobufError,
};

use super::{
    super::Message, P2pNetworkKadIncomingStreamState, P2pNetworkKadOutgoingStreamState,
    P2pNetworkKadStreamState, P2pNetworkKademliaStreamAction,
};

impl P2pNetworkKadIncomingStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String> {
        let (action, _meta) = action.split();

        match (&self, action) {
            (
                P2pNetworkKadIncomingStreamState::Default,
                P2pNetworkKademliaStreamAction::New { incoming, .. },
            ) if *incoming => {
                *self = P2pNetworkKadIncomingStreamState::WaitingForRequest {
                    expect_close: false,
                };
                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::WaitingForRequest { .. },
                P2pNetworkKademliaStreamAction::IncomingData { data, .. },
            ) => {
                let data = &data.0;
                let mut reader = BytesReader::from_bytes(data);

                let Ok(encoded_len) = reader.read_varint32(data).map(|v| v as usize) else {
                    *self = P2pNetworkKadIncomingStreamState::Error(
                        P2pNetworkStreamProtobufError::MessageLength,
                    );
                    return Ok(());
                };

                if encoded_len > limits.kademlia_request() {
                    *self = P2pNetworkKadIncomingStreamState::Error(
                        P2pNetworkStreamProtobufError::Limit(
                            encoded_len,
                            limits.kademlia_request(),
                        ),
                    );
                    return Ok(());
                }

                let remaining_len = reader.len();

                if let Some(remaining_data) = data.get(data.len() - remaining_len..) {
                    if encoded_len > remaining_len {
                        *self = P2pNetworkKadIncomingStreamState::PartialRequestReceived {
                            len: encoded_len,
                            data: remaining_data.to_vec(),
                        };
                        return Ok(());
                    }

                    self.handle_incoming_request(encoded_len, remaining_data)
                } else {
                    *self = P2pNetworkKadIncomingStreamState::Error(
                        P2pNetworkStreamProtobufError::Message("out of bounds".to_owned()),
                    );
                    Ok(())
                }
            }
            (
                P2pNetworkKadIncomingStreamState::PartialRequestReceived { len, data },
                P2pNetworkKademliaStreamAction::IncomingData { data: new_data, .. },
            ) => {
                let mut data = data.clone();
                data.extend_from_slice(&new_data.0);

                if *len > data.len() {
                    *self = P2pNetworkKadIncomingStreamState::PartialRequestReceived {
                        len: *len,
                        data,
                    };
                    return Ok(());
                }

                self.handle_incoming_request(*len, &data)
            }
            (
                P2pNetworkKadIncomingStreamState::RequestIsReady { .. },
                P2pNetworkKademliaStreamAction::WaitOutgoing { .. },
            ) => {
                *self = P2pNetworkKadIncomingStreamState::WaitingForReply;
                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::WaitingForReply,
                P2pNetworkKademliaStreamAction::SendResponse { data, .. },
            ) => {
                let message = Message::from(data);
                let bytes = serialize_into_vec(&message).map_err(|e| format!("{e}"))?;
                *self = P2pNetworkKadIncomingStreamState::ResponseBytesAreReady { bytes };
                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::ResponseBytesAreReady { .. },
                P2pNetworkKademliaStreamAction::WaitIncoming { .. },
            ) => {
                *self = P2pNetworkKadIncomingStreamState::WaitingForRequest { expect_close: true };
                Ok(())
            }
            (
                P2pNetworkKadIncomingStreamState::WaitingForRequest { expect_close, .. },
                P2pNetworkKademliaStreamAction::RemoteClose { .. },
            ) if *expect_close => {
                *self = P2pNetworkKadIncomingStreamState::Closing;
                Ok(())
            }
            _ => Err(format!(
                "kademlia incoming stream state {self:?} is incorrect for action {action:?}",
            )),
        }
    }

    fn handle_incoming_request(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<Message>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkKadIncomingStreamState::Error(
                    P2pNetworkStreamProtobufError::Message(e.to_string()),
                );
                return Ok(());
            }
        };

        let data = match P2pNetworkKademliaRpcRequest::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkKadIncomingStreamState::Error(e.into());
                return Ok(());
            }
        };

        *self = P2pNetworkKadIncomingStreamState::RequestIsReady { data };
        Ok(())
    }
}

impl P2pNetworkKadOutgoingStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String> {
        let (action, _meta) = action.split();
        match (&self, action) {
            (
                P2pNetworkKadOutgoingStreamState::Default,
                P2pNetworkKademliaStreamAction::New { incoming, .. },
            ) if !*incoming => {
                *self = P2pNetworkKadOutgoingStreamState::WaitingForRequest {
                    expect_close: false,
                };
                Ok(())
            }

            (
                P2pNetworkKadOutgoingStreamState::WaitingForRequest { .. },
                P2pNetworkKademliaStreamAction::SendRequest { data, .. },
            ) => {
                let message = Message::from(data);
                let bytes = serialize_into_vec(&message).map_err(|e| format!("{e}"))?;
                *self = P2pNetworkKadOutgoingStreamState::RequestBytesAreReady { bytes };
                Ok(())
            }
            (
                P2pNetworkKadOutgoingStreamState::RequestBytesAreReady { .. },
                P2pNetworkKademliaStreamAction::WaitIncoming { .. },
            ) => {
                *self = P2pNetworkKadOutgoingStreamState::WaitingForReply;
                Ok(())
            }

            (
                P2pNetworkKadOutgoingStreamState::WaitingForReply { .. },
                P2pNetworkKademliaStreamAction::IncomingData { data, .. },
            ) => {
                let data = &data.0;

                let mut reader = BytesReader::from_bytes(data);
                let Ok(encoded_len) = reader.read_varint32(data).map(|v| v as usize) else {
                    *self = P2pNetworkKadOutgoingStreamState::Error(
                        P2pNetworkStreamProtobufError::MessageLength,
                    );
                    return Ok(());
                };

                if encoded_len > limits.kademlia_response() {
                    *self = P2pNetworkKadOutgoingStreamState::Error(
                        P2pNetworkStreamProtobufError::Limit(
                            encoded_len,
                            limits.kademlia_response(),
                        ),
                    );
                    return Ok(());
                }

                let remaining_len = reader.len();

                if let Some(remaining_data) = data.get(data.len() - remaining_len..) {
                    if encoded_len > remaining_len {
                        *self = P2pNetworkKadOutgoingStreamState::PartialReplyReceived {
                            len: encoded_len,
                            data: remaining_data.to_vec(),
                        };
                        return Ok(());
                    }

                    self.handle_incoming_response(encoded_len, remaining_data)
                } else {
                    *self = P2pNetworkKadOutgoingStreamState::Error(
                        P2pNetworkStreamProtobufError::Message("out of bounds".to_owned()),
                    );
                    Ok(())
                }
            }
            (
                P2pNetworkKadOutgoingStreamState::PartialReplyReceived { len, data },
                P2pNetworkKademliaStreamAction::IncomingData { data: new_data, .. },
            ) => {
                let mut data = data.clone();
                data.extend_from_slice(&new_data.0);

                if *len > data.len() {
                    *self =
                        P2pNetworkKadOutgoingStreamState::PartialReplyReceived { len: *len, data };
                    return Ok(());
                }

                self.handle_incoming_response(*len, &data)
            }
            (
                P2pNetworkKadOutgoingStreamState::ResponseIsReady { .. },
                P2pNetworkKademliaStreamAction::WaitOutgoing { .. },
            ) => {
                *self = P2pNetworkKadOutgoingStreamState::WaitingForRequest { expect_close: true };
                Ok(())
            }
            (
                P2pNetworkKadOutgoingStreamState::WaitingForRequest { expect_close },
                P2pNetworkKademliaStreamAction::Close { .. },
            ) if *expect_close => {
                *self =
                    P2pNetworkKadOutgoingStreamState::RequestBytesAreReady { bytes: Vec::new() };
                Ok(())
            }
            (
                P2pNetworkKadOutgoingStreamState::Closing,
                P2pNetworkKademliaStreamAction::RemoteClose { .. },
            ) => {
                *self = P2pNetworkKadOutgoingStreamState::Closed;
                Ok(())
            }
            _ => Err(format!(
                "kademlia outgoing stream state {self:?} is incorrect for action {action:?}",
            )),
        }
    }

    fn handle_incoming_response(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<Message>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkKadOutgoingStreamState::Error(
                    P2pNetworkStreamProtobufError::Message(e.to_string()),
                );
                return Ok(());
            }
        };

        let data = match P2pNetworkKademliaRpcReply::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkKadOutgoingStreamState::Error(e.into());
                return Ok(());
            }
        };

        *self = P2pNetworkKadOutgoingStreamState::ResponseIsReady { data };
        Ok(())
    }
}

impl P2pNetworkKadStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String> {
        match self {
            P2pNetworkKadStreamState::Incoming(i) => i.reducer(action, limits),
            P2pNetworkKadStreamState::Outgoing(o) => o.reducer(action, limits),
        }
    }
}

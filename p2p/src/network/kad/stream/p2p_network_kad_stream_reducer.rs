use quick_protobuf::{serialize_into_vec, BytesReader};
use redux::ActionWithMeta;

use crate::{P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest};

use super::{
    super::Message, P2pNetworkKadStreamKind, P2pNetworkKadStreamState,
    P2pNetworkKademliaStreamAction,
};

impl P2pNetworkKadStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKademliaStreamAction>,
    ) -> Result<(), String> {
        use super::P2pNetworkKadStreamState as S;
        use super::P2pNetworkKademliaStreamAction as A;
        let (action, _meta) = action.split();
        // println!("=== state:  {self:?}");
        // println!("=== action: {action:?}");
        match (&self, action) {
            (S::Default, A::New { incoming, .. }) => {
                let kind = P2pNetworkKadStreamKind::from(*incoming);
                *self = match kind {
                    P2pNetworkKadStreamKind::Incoming => S::WaitingIncoming {
                        kind,
                        expect_data: true,
                        expect_close: false,
                    },
                    P2pNetworkKadStreamKind::Outgoing => S::WaitingOutgoing {
                        kind,
                        expect_close: false,
                    },
                };
                Ok(())
            }
            (
                S::WaitingIncoming {
                    kind, expect_data, ..
                },
                A::IncomingData { data, .. },
            ) if *expect_data => {
                let data = &data.0;

                let mut reader = BytesReader::from_bytes(data);
                let Ok(len) = reader.read_varint32(data).map(|v| v as usize) else {
                    *self = S::Error("error reading message length".to_owned());
                    return Ok(());
                };

                if len > reader.len() {
                    *self = S::IncomingPartialData {
                        kind: *kind,
                        len,
                        data: data[(len - reader.len())..].to_vec(),
                    };
                    return Ok(());
                }

                self.handle_incoming_message(*kind, len, &data[data.len() - reader.len()..])
            }
            (
                S::IncomingPartialData { kind, len, data },
                A::IncomingData { data: new_data, .. },
            ) => {
                let mut data = data.clone();
                data.extend_from_slice(&new_data.0);

                if *len > data.len() {
                    *self = S::IncomingPartialData {
                        kind: *kind,
                        len: *len,
                        data,
                    };
                    return Ok(());
                }

                self.handle_incoming_message(*kind, *len, &data)
            }
            (S::IncomingRequest { .. }, A::WaitOutgoing { .. }) => {
                *self = S::WaitingOutgoing {
                    kind: P2pNetworkKadStreamKind::Incoming,
                    expect_close: false,
                };
                Ok(())
            }
            (S::IncomingReply { .. }, A::WaitOutgoing { .. }) => {
                *self = S::WaitingOutgoing {
                    kind: P2pNetworkKadStreamKind::Outgoing,
                    expect_close: true,
                };
                Ok(())
            }
            (
                S::WaitingOutgoing {
                    kind: kind @ P2pNetworkKadStreamKind::Outgoing,
                    expect_close,
                },
                A::SendRequest { data, .. },
            ) if !*expect_close => {
                let message = Message::from(data);
                let bytes = serialize_into_vec(&message).map_err(|e| format!("{e}"))?;
                *self = S::OutgoingBytes { kind: *kind, bytes };
                Ok(())
            }
            (
                S::WaitingOutgoing {
                    kind: kind @ P2pNetworkKadStreamKind::Incoming,
                    expect_close,
                },
                A::SendReply { data, .. },
            ) if !*expect_close => {
                let message = Message::from(data);
                let bytes = serialize_into_vec(&message).map_err(|e| format!("{e}"))?;
                *self = S::OutgoingBytes { kind: *kind, bytes };
                Ok(())
            }
            (S::WaitingOutgoing { kind, expect_close }, A::Close { .. }) if *expect_close => {
                *self = S::OutgoingBytes {
                    kind: *kind,
                    bytes: Vec::new(),
                };
                Ok(())
            }
            (S::OutgoingBytes { kind, bytes }, A::WaitIncoming { .. }) => {
                let (expect_data, expect_close) = match kind {
                    // for incoming connection, after the first round we expect both new request or close
                    P2pNetworkKadStreamKind::Incoming => (true, true),
                    // for outgoing connection, we expect data and not close iff we sent data
                    P2pNetworkKadStreamKind::Outgoing => (!bytes.is_empty(), bytes.is_empty()),
                };
                *self = S::WaitingIncoming {
                    kind: *kind,
                    expect_data,
                    expect_close,
                };
                Ok(())
            }
            (S::WaitingIncoming { expect_close, .. }, A::RemoteClose { .. }) if *expect_close => {
                *self = S::Closed;
                Ok(())
            }
            _ => {
                Err(format!(
                    "kademlia state {self:?} is incorrect for action {action:?}"
                ))
            }
        }
    }

    fn handle_incoming_message(
        &mut self,
        kind: P2pNetworkKadStreamKind,
        len: usize,
        data: &[u8],
    ) -> Result<(), String> {
        match kind {
            P2pNetworkKadStreamKind::Incoming => {
                self._handle_incoming_message::<P2pNetworkKademliaRpcRequest>(len, data)
            }
            P2pNetworkKadStreamKind::Outgoing => {
                self._handle_incoming_message::<P2pNetworkKademliaRpcReply>(len, data)
            }
        }
    }

    fn _handle_incoming_message<'a, T>(&mut self, len: usize, data: &'a [u8]) -> Result<(), String>
    where
        T: std::fmt::Debug + TryFrom<Message<'a>> + Into<super::P2pNetworkKadStreamState>,
        T::Error: std::error::Error,
    {
        use super::P2pNetworkKadStreamState::*;

        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<Message>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = Error(format!("error reading protobuf message: {e}"));
                return Ok(());
            }
        };

        let data = match T::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = Error(format!("error converting protobuf message: {e}"));
                return Ok(());
            }
        };

        *self = data.into();
        Ok(())
    }
}

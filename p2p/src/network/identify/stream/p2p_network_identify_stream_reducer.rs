use super::{
    P2pNetworkIdentifyStreamAction, P2pNetworkIdentifyStreamKind, P2pNetworkIdentifyStreamState,
};
use crate::{
    network::identify::{pb::Identify, P2pNetworkIdentify},
    P2pLimits, P2pNetworkStreamProtobufError,
};
use openmina_core::bug_condition;
use prost::Message;
use quick_protobuf::BytesReader;
use redux::ActionWithMeta;

impl P2pNetworkIdentifyStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkIdentifyStreamAction>,
        limits: &P2pLimits,
    ) -> Result<(), String> {
        let (action, _meta) = action.split();
        match &self {
            P2pNetworkIdentifyStreamState::Default => {
                let P2pNetworkIdentifyStreamAction::New { incoming, .. } = action else {
                    // enabling conditions should prevent receiving other actions in Default state
                    bug_condition!("Received action {:?} in Default state", action);
                    return Ok(());
                };

                let kind = P2pNetworkIdentifyStreamKind::from(*incoming);

                *self = match kind {
                    // For incoming streams we prepare to send the Identify message
                    P2pNetworkIdentifyStreamKind::Incoming => {
                        P2pNetworkIdentifyStreamState::SendIdentify
                    }
                    // For outgoing streams we expect to get the Identify message from the remote peer
                    P2pNetworkIdentifyStreamKind::Outgoing => {
                        P2pNetworkIdentifyStreamState::RecvIdentify
                    }
                };

                Ok(())
            }
            P2pNetworkIdentifyStreamState::RecvIdentify => match action {
                P2pNetworkIdentifyStreamAction::IncomingData { data, .. } => {
                    let data = &data.0;
                    let mut reader = BytesReader::from_bytes(data);
                    let Ok(len) = reader.read_varint32(data).map(|v| v as usize) else {
                        *self = P2pNetworkIdentifyStreamState::Error(
                            P2pNetworkStreamProtobufError::MessageLength,
                        );
                        return Ok(());
                    };

                    // TODO: implement as configuration option
                    if len > limits.identify_message() {
                        *self = P2pNetworkIdentifyStreamState::Error(
                            P2pNetworkStreamProtobufError::Limit(len, limits.identify_message()),
                        );
                        return Ok(());
                    }

                    let data = &data[(data.len() - reader.len())..];

                    if len > reader.len() {
                        *self = P2pNetworkIdentifyStreamState::IncomingPartialData {
                            len,
                            data: data.to_vec(),
                        };
                        Ok(())
                    } else {
                        self.handle_incoming_identify_message(len, data)
                    }
                }
                P2pNetworkIdentifyStreamAction::RemoteClose { .. } => Ok(()),
                _ => {
                    // State and connection cleanup should be handled by timeout
                    bug_condition!("Received action {:?} in RecvIdentify state", action);
                    Ok(())
                }
            },
            P2pNetworkIdentifyStreamState::IncomingPartialData { len, data } => match action {
                P2pNetworkIdentifyStreamAction::IncomingData { data: new_data, .. } => {
                    let mut data = data.clone();
                    data.extend_from_slice(&new_data.0);

                    if *len > data.len() {
                        *self =
                            P2pNetworkIdentifyStreamState::IncomingPartialData { len: *len, data };
                        Ok(())
                    } else {
                        self.handle_incoming_identify_message(*len, &data)
                    }
                }
                P2pNetworkIdentifyStreamAction::RemoteClose { .. } => Ok(()),
                _ => {
                    // State and connection cleanup should be handled by timeout
                    bug_condition!("Received action {:?} in IncomingPartialData state", action);
                    Ok(())
                }
            },
            P2pNetworkIdentifyStreamState::SendIdentify => match action {
                P2pNetworkIdentifyStreamAction::RemoteClose { .. } => Ok(()),
                P2pNetworkIdentifyStreamAction::Close { .. } => Ok(()),
                _ => {
                    // State and connection cleanup should be handled by timeout
                    bug_condition!("Received action {:?} in SendIdentify state", action);
                    Ok(())
                }
            },
            P2pNetworkIdentifyStreamState::IdentifyReceived { .. } => Ok(()),
            P2pNetworkIdentifyStreamState::Error(_) => {
                // TODO
                Ok(())
            }
        }
    }

    fn handle_incoming_identify_message(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        let message = match Identify::decode(&data[..len]) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkIdentifyStreamState::Error(
                    P2pNetworkStreamProtobufError::Message(e.to_string()),
                );
                return Ok(());
            }
        };

        let data = match P2pNetworkIdentify::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkIdentifyStreamState::Error(e.into());
                return Ok(());
            }
        };

        *self = data.into();
        Ok(())
    }
}

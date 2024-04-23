use super::{
    P2pNetworkIdentifyStreamAction, P2pNetworkIdentifyStreamKind, P2pNetworkIdentifyStreamState,
};
use crate::network::identify::{Identify, P2pNetworkIdentify};
use quick_protobuf::BytesReader;
use redux::ActionWithMeta;

impl P2pNetworkIdentifyStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkIdentifyStreamAction>,
    ) -> Result<(), String> {
        use super::P2pNetworkIdentifyStreamAction as A;
        use super::P2pNetworkIdentifyStreamState as S;
        let (action, _meta) = action.split();
        //println!("=== IDENTIFY state:  {self:?}");
        //println!("=== IDENTIFY action: {action:?}");
        match &self {
            S::Default => {
                if let A::New { incoming, .. } = action {
                    let kind = P2pNetworkIdentifyStreamKind::from(*incoming);

                    *self = match kind {
                        // For incoming streams we prepare to send the Identify message
                        P2pNetworkIdentifyStreamKind::Incoming => S::SendIdentify,
                        // For outgoing streams we expect to get the Identify message from the remote peer
                        P2pNetworkIdentifyStreamKind::Outgoing => S::RecvIdentify,
                    };
                    Ok(())
                } else {
                    // enabling conditions should prevent receiving other actions in Default state
                    unreachable!()
                }
            }
            S::RecvIdentify => match action {
                A::IncomingData { data, .. } => {
                    let data = &data.0;
                    let mut reader = BytesReader::from_bytes(data);
                    let Ok(len) = reader.read_varint32(data).map(|v| v as usize) else {
                        *self = S::Error("error reading message length".to_owned());
                        return Ok(());
                    };

                    // TODO: implement as configuration option
                    if len > 0x1000 {
                        *self = S::Error(format!("Identify message is too long ({})", len));
                        return Ok(());
                    }

                    let data = &data[(data.len() - reader.len())..];

                    if len > reader.len() {
                        *self = S::IncomingPartialData {
                            len,
                            data: data.to_vec(),
                        };
                        Ok(())
                    } else {
                        self.handle_incoming_identify_message(len, data)
                    }
                }
                A::Close { .. } => todo!(),
                _ => unreachable!(),
            },
            S::IncomingPartialData { len, data } => match action {
                A::IncomingData { data: new_data, .. } => {
                    let mut data = data.clone();
                    data.extend_from_slice(&new_data.0);

                    if *len > data.len() {
                        *self = S::IncomingPartialData { len: *len, data };
                        Ok(())
                    } else {
                        self.handle_incoming_identify_message(*len, &data)
                    }
                }
                A::Close { .. } => todo!(),
                _ => unreachable!(),
            },
            S::SendIdentify => match action {
                A::Close { .. } => Ok(()),
                _ => unreachable!(),
            },
            S::IdentifyReceived { .. } => Ok(()),
            S::Error(_) => {
                // TODO
                Ok(())
            }
        }
    }

    fn handle_incoming_identify_message(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<Identify>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkIdentifyStreamState::Error(format!(
                    "error reading protobuf message: {e}"
                ));
                return Ok(());
            }
        };

        let data = match P2pNetworkIdentify::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkIdentifyStreamState::Error(format!(
                    "error converting protobuf message: {e}"
                ));
                return Ok(());
            }
        };

        *self = dbg!(data).into();
        Ok(())
    }
}

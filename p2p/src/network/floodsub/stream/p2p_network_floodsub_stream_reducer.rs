use super::{
    P2pNetworkFloodsubStreamAction, P2pNetworkFloodsubStreamKind, P2pNetworkFloodsubStreamState,
};
use crate::{
    network::floodsub::p2p_network_floodsub_message::RPC, network::floodsub::P2pNetworkFloodsub,
};
use quick_protobuf::BytesReader;
use redux::ActionWithMeta;

impl P2pNetworkFloodsubStreamState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkFloodsubStreamAction>,
    ) -> Result<(), String> {
        use super::P2pNetworkFloodsubStreamAction as A;
        use super::P2pNetworkFloodsubStreamState as S;
        let (action, _meta) = action.split();
        println!("=== FLOODSUB state:  {self:?}");
        println!("=== FLOODSUB action: {action:?}");
        match &self {
            S::Default => {
                if let A::New { incoming, .. } = action {
                    let kind = P2pNetworkFloodsubStreamKind::from(*incoming);

                    *self = match kind {
                        P2pNetworkFloodsubStreamKind::Incoming => S::WaitForInput,
                        P2pNetworkFloodsubStreamKind::Outgoing => S::SendSubscriptions,
                    };
                    Ok(())
                } else {
                    // enabling conditions should prevent receiving other actions in Default state
                    unreachable!()
                }
            }
            S::SendSubscriptions => {
                todo!()
            }
            S::WaitForInput => match action {
                A::IncomingData { data, .. } => {
                    let data = &data.0;
                    let mut reader = BytesReader::from_bytes(data);
                    let Ok(len) = reader.read_varint32(data).map(|v| v as usize) else {
                        *self = S::Error("error reading message length".to_owned());
                        return Ok(());
                    };

                    // TODO: ATM hardcoded to 1 MiB as in spec but should be configurable.
                    if len > 0x100000 {
                        *self = S::Error(format!("message is too long ({})", len));
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
                        self.handle_incoming_message(len, data)
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
                        self.handle_incoming_message(*len, &data)
                    }
                }
                A::Close { .. } => todo!(),
                _ => unreachable!(),
            },
            S::MessageReceived { .. } => Ok(()),
            S::Error(_) => todo!(),
        }
    }

    fn handle_incoming_message(&mut self, len: usize, data: &[u8]) -> Result<(), String> {
        let mut reader = BytesReader::from_bytes(data);

        let message = match reader.read_message_by_len::<RPC>(data, len) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkFloodsubStreamState::Error(format!(
                    "error reading protobuf message: {e}"
                ));
                return Ok(());
            }
        };

        let data = match P2pNetworkFloodsub::try_from(message.clone()) {
            Ok(v) => v,
            Err(e) => {
                *self = P2pNetworkFloodsubStreamState::Error(format!(
                    "error converting protobuf message: {e}"
                ));
                return Ok(());
            }
        };

        *self = data.into();
        Ok(())
    }
}

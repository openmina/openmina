use salsa_simple::XSalsa20;

use super::{
    p2p_network_pnet_state::{Half, P2pNetworkPnetState},
    *,
};

impl Half {
    fn reduce(&mut self, shared_secret: &[u8; 32], data: &[u8]) {
        match self {
            Half::Buffering { buffer, offset } => {
                if *offset + data.len() < 24 {
                    buffer[*offset..(*offset + data.len())].clone_from_slice(data);
                    *offset += data.len();
                } else {
                    if *offset < 24 {
                        buffer[*offset..24].clone_from_slice(&data[..(24 - *offset)]);
                    }
                    let nonce = *buffer;
                    let remaining = data[(24 - *offset)..].to_vec().into_boxed_slice();
                    *self = Half::Done {
                        cipher: XSalsa20::new(*shared_secret, nonce),
                        to_send: vec![],
                    };
                    self.reduce(shared_secret, &remaining);
                }
            }
            Half::Done { cipher, to_send } => {
                *to_send = data.to_vec();
                cipher.apply_keystream(to_send.as_mut());
            }
        }
    }
}

impl P2pNetworkPnetState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkPnetAction>) {
        match action.action() {
            P2pNetworkPnetAction::IncomingData { data, .. } => {
                self.incoming.reduce(&self.shared_secret, data);
            }
            P2pNetworkPnetAction::OutgoingData { data, .. } => {
                self.outgoing.reduce(&self.shared_secret, data)
            }
            P2pNetworkPnetAction::SetupNonce { nonce, .. } => {
                self.outgoing.reduce(&self.shared_secret, nonce)
            }
        }
    }
}

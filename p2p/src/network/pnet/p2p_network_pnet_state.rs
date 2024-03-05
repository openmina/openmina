use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use salsa_simple::XSalsa20;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetState {
    pub shared_secret: [u8; 32],

    pub incoming: Half,
    pub outgoing: Half,
}

impl Drop for P2pNetworkPnetState {
    fn drop(&mut self) {
        self.shared_secret.zeroize();
    }
}

impl P2pNetworkPnetState {
    pub fn new(shared_secret: [u8; 32]) -> Self {
        P2pNetworkPnetState {
            shared_secret,

            incoming: Half::Buffering {
                buffer: [0; 24],
                offset: 0,
            },
            outgoing: Half::Buffering {
                buffer: [0; 24],
                offset: 0,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Half {
    Buffering { buffer: [u8; 24], offset: usize },
    Done { cipher: XSalsa20, to_send: Vec<u8> },
}

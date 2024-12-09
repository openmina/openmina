use malloc_size_of_derive::MallocSizeOf;
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use salsa_simple::XSalsa20;

use crate::P2pTimeouts;

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub struct P2pNetworkPnetState {
    pub time: Option<Timestamp>,

    #[serde_as(as = "serde_with::hex::Hex")]
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
    pub fn new(shared_secret: [u8; 32], time: Timestamp) -> Self {
        P2pNetworkPnetState {
            time: Some(time),
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

    pub fn is_timed_out(&self, now: Timestamp, timeouts: &P2pTimeouts) -> bool {
        if matches!(self.incoming, Half::Buffering { .. })
            || matches!(self.outgoing, Half::Buffering { .. })
        {
            if let Some(time) = self.time {
                now.checked_sub(time)
                    .and_then(|dur| timeouts.pnet.map(|to| dur >= to))
                    .unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Half {
    Buffering { buffer: [u8; 24], offset: usize },
    Done { cipher: XSalsa20, to_send: Vec<u8> },
}

mod measurement {
    use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

    use super::*;

    impl MallocSizeOf for Half {
        fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
            match self {
                Self::Done { to_send, .. } => to_send.capacity(),
                _ => 0,
            }
        }
    }
}

use serde::{Deserialize, Serialize};

const MAX_TOKEN_LENGTH: usize = 256;

/// Possible valid token of multistream-select protocol
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Token {
    Handshake,
    Na,
    SimultaneousConnect,
    Protocol(Protocol),
}

impl Token {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Handshake => b"\x13/multistream/1.0.0\n",
            Self::Na => b"\x03na\n",
            Self::SimultaneousConnect => b"\x1d/libp2p/simultaneous-connect\n",
            Self::Protocol(v) => v.name(),
        }
    }

    // WARNING: don't forget to add net protocol here
    pub const ALL: &'static [Self] = &[
        Token::Handshake,
        Token::Na,
        Token::SimultaneousConnect,
        Token::Protocol(Protocol::Auth(AuthKind::Noise)),
        Token::Protocol(Protocol::Mux(MuxKind::Yamux1_0_0)),
        Token::Protocol(Protocol::Stream(StreamKind::Discovery(
            DiscoveryAlgorithm::Kademlia1_0_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Broadcast(
            BroadcastAlgorithm::Meshsub1_1_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1))),
    ];
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Protocol {
    Auth(AuthKind),
    Mux(MuxKind),
    Stream(StreamKind),
}

impl Protocol {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Auth(v) => v.name(),
            Self::Mux(v) => v.name(),
            Self::Stream(v) => v.name(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum AuthKind {
    Noise,
}

impl AuthKind {
    pub const fn name(&self) -> &'static [u8] {
        b"\x07/noise\n"
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum MuxKind {
    Yamux1_0_0,
}

impl MuxKind {
    pub const fn name(&self) -> &'static [u8] {
        b"\x12/coda/yamux/1.0.0\n"
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum StreamKind {
    Discovery(DiscoveryAlgorithm),
    Broadcast(BroadcastAlgorithm),
    Rpc(RpcAlgorithm),
}

impl StreamKind {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Discovery(v) => v.name(),
            Self::Broadcast(v) => v.name(),
            Self::Rpc(v) => v.name(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum DiscoveryAlgorithm {
    Kademlia1_0_0,
}

impl DiscoveryAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Kademlia1_0_0 => b"\x10/coda/kad/1.0.0\n",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum BroadcastAlgorithm {
    Meshsub1_1_0,
}

impl BroadcastAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Meshsub1_1_0 => b"\x0f/meshsub/1.1.0\n",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum RpcAlgorithm {
    Rpc0_0_1,
}

impl RpcAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Rpc0_0_1 => b"\x10coda/rpcs/0.0.1\n",
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct State {
    buffer: Vec<u8>,
}

impl State {
    pub fn put(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// return protocol name followed by the rest of unparsed data
    /// use `State::consume` after return some parsed token
    pub fn parse_token(&mut self) -> Result<Option<Token>, ()> {
        use unsigned_varint::decode::{self, Error::*};

        let (len, rem) = match decode::usize(&self.buffer) {
            Ok(v) => v,
            Err(Insufficient) => return Ok(None),
            Err(_) => return Err(()),
        };
        let len_length = self.buffer.len() - rem.len();
        if len_length > MAX_TOKEN_LENGTH {
            return Err(());
        }

        if self.buffer.len() < len_length + len {
            return Ok(None);
        }

        if let Some(token) = Token::ALL
            .iter()
            .find(|t| t.name() == &self.buffer[..(len_length + len)])
        {
            self.buffer.drain(..(len_length + len));

            Ok(Some(*token))
        } else {
            Err(())
        }
    }
}

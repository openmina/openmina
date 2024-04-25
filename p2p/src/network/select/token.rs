use serde::{Deserialize, Serialize};

const MAX_TOKEN_LENGTH: usize = 256;

/// Possible valid token of multistream-select protocol
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Token {
    Handshake,
    Na,
    SimultaneousConnect,
    Protocol(Protocol),
    UnknownProtocol(Vec<u8>),
}

impl Token {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Handshake => b"\x13/multistream/1.0.0\n",
            Self::Na => b"\x03na\n",
            Self::SimultaneousConnect => b"\x1d/libp2p/simultaneous-connect\n",
            Self::Protocol(v) => v.name(),
            Self::UnknownProtocol(_) => b"",
        }
    }

    // WARNING: don't forget to add net protocol here
    pub const ALL: &'static [Self] = &[
        Token::Handshake,
        Token::Na,
        Token::SimultaneousConnect,
        Token::Protocol(Protocol::Auth(AuthKind::Noise)),
        Token::Protocol(Protocol::Mux(MuxKind::Yamux1_0_0)),
        Token::Protocol(Protocol::Mux(MuxKind::YamuxNoNewLine1_0_0)),
        Token::Protocol(Protocol::Stream(StreamKind::Status(
            StatusAlgorithm::MinaNodeStatus,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Bitswap(
            BitswapAlgorithm::MinaBitswap,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Bitswap(
            BitswapAlgorithm::MinaBitswap1_0_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Bitswap(
            BitswapAlgorithm::MinaBitswap1_1_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Bitswap(
            BitswapAlgorithm::MinaBitswap1_2_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Ping(PingAlgorithm::Ping1_0_0))),
        Token::Protocol(Protocol::Stream(StreamKind::Identify(
            IdentifyAlgorithm::Identify1_0_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Identify(
            IdentifyAlgorithm::IdentifyPush1_0_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Discovery(
            DiscoveryAlgorithm::Kademlia1_0_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Broadcast(
            BroadcastAlgorithm::Floodsub1_0_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Broadcast(
            BroadcastAlgorithm::Meshsub1_0_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Broadcast(
            BroadcastAlgorithm::Meshsub1_1_0,
        ))),
        Token::Protocol(Protocol::Stream(StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1))),
    ];
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
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

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::Auth(v) => v.name_str(),
            Self::Mux(v) => v.name_str(),
            Self::Stream(v) => v.name_str(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AuthKind {
    Noise,
}

impl AuthKind {
    pub const fn name(&self) -> &'static [u8] {
        b"\x07/noise\n"
    }

    pub const fn name_str(&self) -> &'static str {
        "/noise"
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum MuxKind {
    Yamux1_0_0,
    YamuxNoNewLine1_0_0,
}

impl MuxKind {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Yamux1_0_0 => b"\x12/coda/yamux/1.0.0\n",
            Self::YamuxNoNewLine1_0_0 => b"\x11/coda/yamux/1.0.0",
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::Yamux1_0_0 => "/coda/yamux/1.0.0",
            Self::YamuxNoNewLine1_0_0 => "/coda/yamux/1.0.0",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum StreamKind {
    Status(StatusAlgorithm),
    Bitswap(BitswapAlgorithm),
    Ping(PingAlgorithm),
    Identify(IdentifyAlgorithm),
    Discovery(DiscoveryAlgorithm),
    Broadcast(BroadcastAlgorithm),
    Rpc(RpcAlgorithm),
}

impl StreamKind {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Status(v) => v.name(),
            Self::Bitswap(v) => v.name(),
            Self::Ping(v) => v.name(),
            Self::Identify(v) => v.name(),
            Self::Discovery(v) => v.name(),
            Self::Broadcast(v) => v.name(),
            Self::Rpc(v) => v.name(),
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::Status(v) => v.name_str(),
            Self::Bitswap(v) => v.name_str(),
            Self::Ping(v) => v.name_str(),
            Self::Identify(v) => v.name_str(),
            Self::Discovery(v) => v.name_str(),
            Self::Broadcast(v) => v.name_str(),
            Self::Rpc(v) => v.name_str(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum StatusAlgorithm {
    MinaNodeStatus,
}

impl StatusAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::MinaNodeStatus => b"\x12/mina/node-status\n",
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::MinaNodeStatus => "/mina/node-status",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum BitswapAlgorithm {
    MinaBitswap,
    MinaBitswap1_0_0,
    MinaBitswap1_1_0,
    MinaBitswap1_2_0,
}

impl BitswapAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::MinaBitswap => b"\x24/mina/bitswap-exchange/ipfs/bitswap\n",
            Self::MinaBitswap1_0_0 => b"\x2a/mina/bitswap-exchange/ipfs/bitswap/1.0.0\n",
            Self::MinaBitswap1_1_0 => b"\x2a/mina/bitswap-exchange/ipfs/bitswap/1.1.0\n",
            Self::MinaBitswap1_2_0 => b"\x2a/mina/bitswap-exchange/ipfs/bitswap/1.2.0\n",
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::MinaBitswap => "/mina/bitswap-exchange/ipfs/bitswap",
            Self::MinaBitswap1_0_0 => "/mina/bitswap-exchange/ipfs/bitswap/1.0.0",
            Self::MinaBitswap1_1_0 => "/mina/bitswap-exchange/ipfs/bitswap/1.1.0",
            Self::MinaBitswap1_2_0 => "/mina/bitswap-exchange/ipfs/bitswap/1.2.0",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum PingAlgorithm {
    Ping1_0_0,
}

impl PingAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Ping1_0_0 => b"\x11/ipfs/ping/1.0.0\n",
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::Ping1_0_0 => "/ipfs/ping/1.0.0",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum IdentifyAlgorithm {
    Identify1_0_0,
    IdentifyPush1_0_0,
}

impl IdentifyAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Identify1_0_0 => b"\x0f/ipfs/id/1.0.0\n",
            Self::IdentifyPush1_0_0 => b"\x14/ipfs/id/push/1.0.0\n",
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::Identify1_0_0 => "/ipfs/id/1.0.0",
            Self::IdentifyPush1_0_0 => "/ipfs/id/push/1.0.0",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum DiscoveryAlgorithm {
    Kademlia1_0_0,
}

impl DiscoveryAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Kademlia1_0_0 => b"\x10/coda/kad/1.0.0\n",
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::Kademlia1_0_0 => "/coda/kad/1.0.0",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum BroadcastAlgorithm {
    Floodsub1_0_0,
    Meshsub1_0_0,
    Meshsub1_1_0,
}

impl BroadcastAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Floodsub1_0_0 => b"\x10/floodsub/1.0.0\n",
            Self::Meshsub1_0_0 => b"\x0f/meshsub/1.0.0\n",
            Self::Meshsub1_1_0 => b"\x0f/meshsub/1.1.0\n",
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::Floodsub1_0_0 => "/floodsub/1.0.0",
            Self::Meshsub1_0_0 => "/meshsub/1.0.0",
            Self::Meshsub1_1_0 => "/meshsub/1.1.0",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum RpcAlgorithm {
    Rpc0_0_1,
}

impl RpcAlgorithm {
    pub const fn name(&self) -> &'static [u8] {
        match self {
            Self::Rpc0_0_1 => b"\x10coda/rpcs/0.0.1\n",
        }
    }

    pub const fn name_str(&self) -> &'static str {
        match self {
            Self::Rpc0_0_1 => "coda/rpcs/0.0.1",
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct State {
    buffer: Vec<u8>,
}

pub struct ParseTokenError;

impl State {
    pub fn put(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// return protocol name followed by the rest of unparsed data
    /// use `State::consume` after return some parsed token
    pub fn parse_token(&mut self) -> Result<Option<Token>, ParseTokenError> {
        use unsigned_varint::decode::{self, Error::*};

        let (len, rem) = match decode::usize(&self.buffer) {
            Ok(v) => v,
            Err(Insufficient) => return Ok(None),
            Err(_) => return Err(ParseTokenError),
        };
        let len_length = self.buffer.len() - rem.len();
        if len_length > MAX_TOKEN_LENGTH {
            return Err(ParseTokenError);
        }

        if self.buffer.len() < len_length + len {
            return Ok(None);
        }

        // buffer content should match one of tokens
        let token = Token::ALL
            .iter()
            .find(|t| t.name() == &self.buffer[..(len_length + len)]);
        let name = self.buffer.drain(..(len_length + len)).collect();
        Ok(Some(token.cloned().unwrap_or(Token::UnknownProtocol(name))))
    }
}

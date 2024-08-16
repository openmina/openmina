pub mod best_tip;
pub mod rpc;
pub mod snark;
pub mod snark_job_commitment;
pub mod streaming_rpc;
pub mod transaction;

mod p2p_channels_state;
pub use p2p_channels_state::*;

mod p2p_channels_actions;
pub use p2p_channels_actions::*;

mod p2p_channels_reducer;

mod p2p_channels_effects;

mod p2p_channels_service;
pub use p2p_channels_service::*;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::From;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use self::best_tip::BestTipPropagationChannelMsg;
use self::rpc::RpcChannelMsg;
use self::snark::SnarkPropagationChannelMsg;
use self::snark_job_commitment::SnarkJobCommitmentPropagationChannelMsg;
use self::streaming_rpc::StreamingRpcChannelMsg;
use self::transaction::TransactionPropagationChannelMsg;

#[derive(Serialize, Deserialize, EnumIter, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ChannelId {
    BestTipPropagation = 2,
    TransactionPropagation = 3,
    SnarkPropagation = 4,
    SnarkJobCommitmentPropagation = 5,
    Rpc = 100,
    StreamingRpc = 101,
}

impl ChannelId {
    #[inline(always)]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::BestTipPropagation => "best_tip/propagation",
            Self::TransactionPropagation => "transaction/propagation",
            Self::SnarkPropagation => "snark/propagation",
            Self::SnarkJobCommitmentPropagation => "snark_job_commitment/propagation",
            Self::Rpc => "rpc",
            Self::StreamingRpc => "rpc/streaming",
        }
    }

    pub fn supported_by_libp2p(self) -> bool {
        match self {
            Self::BestTipPropagation => true,
            Self::TransactionPropagation => true,
            Self::SnarkPropagation => true,
            Self::SnarkJobCommitmentPropagation => false,
            Self::Rpc => true,
            Self::StreamingRpc => false,
        }
    }

    pub fn max_msg_size(self) -> usize {
        match self {
            // TODO(binier): reduce this value once we change message for best tip
            // propagation to just propagating consensus state with block hash.
            Self::BestTipPropagation => 32 * 1024 * 1024, // 32MB
            Self::TransactionPropagation => 1024,         // 1KB - just transaction info.
            Self::SnarkPropagation => 1024,               // 1KB - just snark info.
            Self::SnarkJobCommitmentPropagation => 2 * 1024, // 2KB,
            Self::Rpc => 256 * 1024 * 1024,               // 256MB,
            Self::StreamingRpc => 16 * 1024 * 1024,       // 16MB,
        }
    }

    pub fn iter_all() -> impl Iterator<Item = ChannelId> {
        <Self as strum::IntoEnumIterator>::iter()
    }

    pub fn for_libp2p() -> impl Iterator<Item = ChannelId> {
        Self::iter_all().filter(|chan| chan.supported_by_libp2p())
    }
}

impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_u8())
    }
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct MsgId(u64);

impl MsgId {
    pub fn first() -> Self {
        Self(1)
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, From, Debug, Clone)]
pub enum ChannelMsg {
    BestTipPropagation(BestTipPropagationChannelMsg),
    TransactionPropagation(TransactionPropagationChannelMsg),
    SnarkPropagation(SnarkPropagationChannelMsg),
    SnarkJobCommitmentPropagation(SnarkJobCommitmentPropagationChannelMsg),
    Rpc(RpcChannelMsg),
    StreamingRpc(StreamingRpcChannelMsg),
}

impl ChannelMsg {
    pub fn channel_id(&self) -> ChannelId {
        match self {
            Self::BestTipPropagation(_) => ChannelId::BestTipPropagation,
            Self::TransactionPropagation(_) => ChannelId::TransactionPropagation,
            Self::SnarkPropagation(_) => ChannelId::SnarkPropagation,
            Self::SnarkJobCommitmentPropagation(_) => ChannelId::SnarkJobCommitmentPropagation,
            Self::Rpc(_) => ChannelId::Rpc,
            Self::StreamingRpc(_) => ChannelId::StreamingRpc,
        }
    }

    pub fn encode<W>(&self, w: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Self::BestTipPropagation(v) => v.binprot_write(w),
            Self::TransactionPropagation(v) => v.binprot_write(w),
            Self::SnarkPropagation(v) => v.binprot_write(w),
            Self::SnarkJobCommitmentPropagation(v) => v.binprot_write(w),
            Self::Rpc(v) => v.binprot_write(w),
            Self::StreamingRpc(v) => v.binprot_write(w),
        }
    }

    pub fn decode<R>(r: &mut R, id: ChannelId) -> Result<Self, binprot::Error>
    where
        Self: Sized,
        R: std::io::Read + ?Sized,
    {
        match id {
            ChannelId::BestTipPropagation => {
                BestTipPropagationChannelMsg::binprot_read(r).map(|v| v.into())
            }
            ChannelId::TransactionPropagation => {
                TransactionPropagationChannelMsg::binprot_read(r).map(|v| v.into())
            }
            ChannelId::SnarkPropagation => {
                SnarkPropagationChannelMsg::binprot_read(r).map(|v| v.into())
            }
            ChannelId::SnarkJobCommitmentPropagation => {
                SnarkJobCommitmentPropagationChannelMsg::binprot_read(r).map(|v| v.into())
            }
            ChannelId::Rpc => RpcChannelMsg::binprot_read(r).map(|v| v.into()),
            ChannelId::StreamingRpc => StreamingRpcChannelMsg::binprot_read(r).map(|v| v.into()),
        }
    }
}

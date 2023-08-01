use shared::snark::Snark;

use crate::PeerId;

use super::{ChannelId, ChannelMsg, MsgId};

pub trait P2pChannelsService: redux::Service {
    fn channel_open(&mut self, peer_id: PeerId, id: ChannelId);
    fn channel_send(&mut self, peer_id: PeerId, msg_id: MsgId, msg: ChannelMsg);
    fn libp2p_broadcast_snark(&mut self, snark: Snark);
}

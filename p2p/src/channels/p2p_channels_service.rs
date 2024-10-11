use crate::{
    identity::{EncryptableType, PublicKey},
    PeerId,
};

use super::{ChannelId, ChannelMsg, MsgId};

pub trait P2pChannelsService: redux::Service {
    fn channel_open(&mut self, peer_id: PeerId, id: ChannelId);
    fn channel_send(&mut self, peer_id: PeerId, msg_id: MsgId, msg: ChannelMsg);
    fn encrypt<T: EncryptableType>(
        &mut self,
        other_pk: &PublicKey,
        message: &T,
    ) -> Result<T::Encrypted, ()>;
    fn decrypt<T: EncryptableType>(
        &mut self,
        other_pk: &PublicKey,
        encrypted: &T::Encrypted,
    ) -> Result<T, ()>;
}

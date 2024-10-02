use redux::ActionMeta;

use crate::channels::{
    snark_job_commitment::{
        P2pChannelsSnarkJobCommitmentAction, SnarkJobCommitmentPropagationChannelMsg,
    },
    ChannelId, MsgId, P2pChannelsService,
};

use super::p2p_channels_snark_job_commitment_effectful_actions::P2pChannelsSnarkJobCommitmentEffectfulAction;

impl P2pChannelsSnarkJobCommitmentEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsSnarkJobCommitmentEffectfulAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::SnarkJobCommitmentPropagation);
                store.dispatch(P2pChannelsSnarkJobCommitmentAction::Pending { peer_id });
            }
            P2pChannelsSnarkJobCommitmentEffectfulAction::RequestSend { peer_id, limit } => {
                let msg = SnarkJobCommitmentPropagationChannelMsg::GetNext { limit };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsSnarkJobCommitmentEffectfulAction::ResponseSend {
                peer_id,
                commitments,
            } => {
                if commitments.is_empty() {
                    return;
                }

                let msg = SnarkJobCommitmentPropagationChannelMsg::WillSend {
                    count: commitments.len() as u8,
                };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());

                for commitment in commitments {
                    let msg = SnarkJobCommitmentPropagationChannelMsg::Commitment(commitment);
                    store
                        .service()
                        .channel_send(peer_id, MsgId::first(), msg.into());
                }
            }
        }
    }
}

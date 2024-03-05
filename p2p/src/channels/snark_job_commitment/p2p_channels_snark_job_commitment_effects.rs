use redux::ActionMeta;

use crate::channels::{ChannelId, MsgId, P2pChannelsService};

use super::{P2pChannelsSnarkJobCommitmentAction, SnarkJobCommitmentPropagationChannelMsg};

impl P2pChannelsSnarkJobCommitmentAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsSnarkJobCommitmentAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::SnarkJobCommitmentPropagation);
                store.dispatch(P2pChannelsSnarkJobCommitmentAction::Pending { peer_id });
            }
            P2pChannelsSnarkJobCommitmentAction::Ready { peer_id } => {
                let limit = 16;
                store.dispatch(P2pChannelsSnarkJobCommitmentAction::RequestSend { peer_id, limit });
            }
            P2pChannelsSnarkJobCommitmentAction::RequestSend { peer_id, limit } => {
                let msg = SnarkJobCommitmentPropagationChannelMsg::GetNext { limit };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsSnarkJobCommitmentAction::Received { peer_id, .. } => {
                let limit = 16;
                store.dispatch(P2pChannelsSnarkJobCommitmentAction::RequestSend { peer_id, limit });
            }
            P2pChannelsSnarkJobCommitmentAction::ResponseSend {
                peer_id,
                commitments,
                ..
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
            P2pChannelsSnarkJobCommitmentAction::Pending { .. } => {}
            P2pChannelsSnarkJobCommitmentAction::PromiseReceived { .. } => {}
            P2pChannelsSnarkJobCommitmentAction::RequestReceived { .. } => {}
        }
    }
}

use openmina_core::transaction::Transaction;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{channels::P2pChannelsAction, P2pState, PeerId};

use super::{P2pChannelsTransactionState, TransactionInfo, TransactionPropagationState};

pub type P2pChannelsTransactionActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pChannelsTransactionAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsTransactionAction {
    Init {
        peer_id: PeerId,
    },
    Pending {
        peer_id: PeerId,
    },
    Ready {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
        limit: u8,
    },
    PromiseReceived {
        peer_id: PeerId,
        promised_count: u8,
    },
    Received {
        peer_id: PeerId,
        transaction: Box<TransactionInfo>,
    },
    RequestReceived {
        peer_id: PeerId,
        limit: u8,
    },
    ResponseSend {
        peer_id: PeerId,
        transactions: Vec<TransactionInfo>,
        first_index: u64,
        last_index: u64,
    },
    Libp2pReceived {
        peer_id: PeerId,
        transaction: Box<Transaction>,
        nonce: u32,
    },
    Libp2pBroadcast {
        transaction: Box<Transaction>,
        nonce: u32,
    },
}

impl P2pChannelsTransactionAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::Init { peer_id }
            | Self::Pending { peer_id }
            | Self::Ready { peer_id }
            | Self::RequestSend { peer_id, .. }
            | Self::PromiseReceived { peer_id, .. }
            | Self::Received { peer_id, .. }
            | Self::RequestReceived { peer_id, .. }
            | Self::ResponseSend { peer_id, .. } => Some(peer_id),
            Self::Libp2pReceived { peer_id, .. } => Some(peer_id),
            Self::Libp2pBroadcast { .. } => None,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pChannelsTransactionAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pChannelsTransactionAction::Init { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.transaction,
                        P2pChannelsTransactionState::Enabled
                    )
                })
            }
            P2pChannelsTransactionAction::Pending { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.transaction,
                        P2pChannelsTransactionState::Init { .. }
                    )
                })
            }
            P2pChannelsTransactionAction::Ready { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.transaction,
                        P2pChannelsTransactionState::Pending { .. }
                    )
                })
            }
            P2pChannelsTransactionAction::RequestSend { peer_id, .. } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.transaction,
                        P2pChannelsTransactionState::Ready {
                            local: TransactionPropagationState::WaitingForRequest { .. }
                                | TransactionPropagationState::Responded { .. },
                            ..
                        }
                    )
                })
            }
            P2pChannelsTransactionAction::PromiseReceived {
                peer_id,
                promised_count,
            } => state.get_ready_peer(peer_id).map_or(false, |p| {
                matches!(
                    &p.channels.transaction,
                    P2pChannelsTransactionState::Ready {
                        local: TransactionPropagationState::Requested {
                            requested_limit, ..
                        }, ..
                    } if *promised_count > 0 && promised_count <= requested_limit
                )
            }),
            P2pChannelsTransactionAction::Received { peer_id, .. } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.transaction,
                        P2pChannelsTransactionState::Ready {
                            local: TransactionPropagationState::Responding { .. },
                            ..
                        }
                    )
                })
            }
            P2pChannelsTransactionAction::RequestReceived { peer_id, limit } => {
                *limit > 0
                    && state.get_ready_peer(peer_id).map_or(false, |p| {
                        matches!(
                            &p.channels.transaction,
                            P2pChannelsTransactionState::Ready {
                                remote: TransactionPropagationState::WaitingForRequest { .. }
                                    | TransactionPropagationState::Responded { .. },
                                ..
                            }
                        )
                    })
            }
            P2pChannelsTransactionAction::ResponseSend {
                peer_id,
                transactions,
                first_index,
                last_index,
            } => {
                !transactions.is_empty()
                    && first_index < last_index
                    && state.get_ready_peer(peer_id).map_or(false, |p| {
                        match &p.channels.transaction {
                            P2pChannelsTransactionState::Ready {
                                remote,
                                next_send_index,
                                ..
                            } => {
                                if first_index < next_send_index {
                                    return false;
                                }
                                match remote {
                                    TransactionPropagationState::Requested {
                                        requested_limit,
                                        ..
                                    } => transactions.len() <= *requested_limit as usize,
                                    _ => false,
                                }
                            }
                            _ => false,
                        }
                    })
            }
            P2pChannelsTransactionAction::Libp2pReceived { peer_id, .. } => {
                cfg!(feature = "p2p-libp2p")
                    && state
                        .peers
                        .get(peer_id)
                        .filter(|p| p.is_libp2p())
                        .and_then(|p| p.status.as_ready())
                        .map_or(false, |p| p.channels.transaction.is_ready())
            }
            P2pChannelsTransactionAction::Libp2pBroadcast { .. } => {
                cfg!(feature = "p2p-libp2p")
                    && state
                        .peers
                        .iter()
                        .any(|(_, p)| p.is_libp2p() && p.status.as_ready().is_some())
            }
        }
    }
}

impl From<P2pChannelsTransactionAction> for crate::P2pAction {
    fn from(action: P2pChannelsTransactionAction) -> Self {
        Self::Channels(P2pChannelsAction::Transaction(action))
    }
}

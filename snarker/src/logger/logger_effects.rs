use crate::p2p::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use crate::p2p::channels::P2pChannelsAction;
use crate::p2p::connection::incoming::P2pConnectionIncomingAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingAction;
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::P2pAction;
use crate::{Action, ActionWithMetaRef, Service, Store};

pub fn logger_effects<S: Service>(_store: &Store<S>, action: ActionWithMetaRef<'_>) {
    let (action, meta) = action.split();
    dbg!(action);
    dbg!(_store.state());

    match action {
        Action::P2p(action) => match action {
            P2pAction::Connection(action) => match action {
                P2pConnectionAction::Outgoing(action) => match action {
                    P2pConnectionOutgoingAction::Init(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerConnectionOutgoingInit",
                            summary = format!("peer_id: {}", action.opts.peer_id()),
                            peer_id = action.opts.peer_id().to_string(),
                            transport = action.opts.kind(),
                        );
                    }
                    P2pConnectionOutgoingAction::Reconnect(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerReconnect",
                            summary = format!("peer_id: {}", action.opts.peer_id()),
                            peer_id = action.opts.peer_id().to_string(),
                            transport = action.opts.kind(),
                        );
                    }
                    P2pConnectionOutgoingAction::Error(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerConnectionOutgoingError",
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionOutgoingAction::Success(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerConnectionOutgoingSuccess",
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    _ => {}
                },
                P2pConnectionAction::Incoming(action) => match action {
                    P2pConnectionIncomingAction::Init(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerConnectionIncomingInit",
                            summary = format!("peer_id: {}", action.opts.peer_id),
                            peer_id = action.opts.peer_id.to_string(),
                            signaling = format!("{:?}", action.opts.signaling),
                        );
                    }
                    P2pConnectionIncomingAction::Error(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerConnectionIncomingError",
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionIncomingAction::Success(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerConnectionOutgoingSuccess",
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    _ => {}
                },
            },
            P2pAction::Disconnection(action) => match action {
                _ => {}
            },
            P2pAction::PeerReady(_) => {}
            P2pAction::Channels(action) => match action {
                P2pChannelsAction::MessageReceived(_) => {}
                P2pChannelsAction::SnarkJobCommitment(action) => match action {
                    P2pChannelsSnarkJobCommitmentAction::Init(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = "PeerChannelsSnarkJobCommitmentInit",
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkJobCommitmentAction::Ready(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = "PeerChannelsSnarkJobCommitmentReady",
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    _ => {}
                },
            },
        },
        _ => {}
    }
}

use redux::ActionMeta;

use crate::ledger::hash_node_at_depth;
use crate::Store;

use super::{
    TransitionFrontierSyncLedgerSnarkedAction, TransitionFrontierSyncLedgerSnarkedService,
};

impl TransitionFrontierSyncLedgerSnarkedAction {
    pub fn effects<S>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: redux::Service + TransitionFrontierSyncLedgerSnarkedService,
    {
        match self {
            TransitionFrontierSyncLedgerSnarkedAction::Pending => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery => {}

            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsInit { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsPending { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsRetry { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsError { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsSuccess { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsReceived { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsAccepted { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsSuccess { .. } => {}

            TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncPending => {}
            TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncSuccess => {}

            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressInit { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressRetry { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess { .. } => {}

            TransitionFrontierSyncLedgerSnarkedAction::P2pDisconnection { .. } => {}

            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived {
                address,
                hashes: (left_hash, right_hash),
                sender,
                ..
            } => {
                // TODO(refactor): service call must be split
                let Some((snarked_ledger_hash, parent_hash)) = None.or_else(|| {
                    let snarked_ledger =
                        store.state().transition_frontier.sync.ledger()?.snarked()?;
                    let parent_hash = snarked_ledger
                        .fetch_pending()?
                        .get(address)?
                        .expected_hash
                        .clone();
                    Some((snarked_ledger.ledger_hash().clone(), parent_hash))
                }) else {
                    return;
                };

                let (Ok(left_hash_fp), Ok(right_hash_fp), Ok(parent_hash_fp)) = (
                    left_hash.to_field(),
                    right_hash.to_field(),
                    parent_hash.to_field(),
                ) else {
                    // Reject in case of invalid fields
                    store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::ChildHashesRejected {
                            address: address.clone(),
                            hashes: (left_hash.clone(), right_hash.clone()),
                            sender: *sender,
                        },
                    );
                    return;
                };

                let actual_hash = hash_node_at_depth(address.length(), left_hash_fp, right_hash_fp);
                if actual_hash != parent_hash_fp {
                    store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::ChildHashesRejected {
                            address: address.clone(),
                            hashes: (left_hash.clone(), right_hash.clone()),
                            sender: *sender,
                        },
                    );

                    return;
                }

                // TODO: for async ledger this needs an intermediary action
                let (previous_left_hash, previous_right_hash) = store
                    .service()
                    .child_hashes_get(snarked_ledger_hash, address)
                    .unwrap();

                store.dispatch(
                    TransitionFrontierSyncLedgerSnarkedAction::ChildHashesAccepted {
                        address: address.clone(),
                        hashes: (left_hash.clone(), right_hash.clone()),
                        previous_hashes: (previous_left_hash, previous_right_hash),
                        sender: *sender,
                    },
                );
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesAccepted { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesRejected { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived {
                address,
                accounts,
                sender,
            } => {
                // TODO(refactor): service call must be split
                let Some((snarked_ledger_hash, parent_hash)) = None.or_else(|| {
                    let snarked_ledger =
                        store.state().transition_frontier.sync.ledger()?.snarked()?;
                    Some((
                        snarked_ledger.ledger_hash().clone(),
                        snarked_ledger
                            .fetch_pending()?
                            .get(address)?
                            .expected_hash
                            .clone(),
                    ))
                }) else {
                    return;
                };

                // After setting the accounts, we get the new computed hash.
                // It must be equal to the parent node hash, otherwise we got
                // bad data from the peer.
                let computed_hash = store
                    .service
                    .accounts_set(snarked_ledger_hash.clone(), address, accounts.clone())
                    .unwrap();

                if computed_hash != parent_hash {
                    store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsRejected {
                            address: address.clone(),
                            sender: *sender,
                        },
                    );
                    return;
                }

                // Setting accounts doesn't immediately compute the hashes in the merkle tree,
                // so we force that here before continuing.
                let compute_hashes_result = store
                    .service
                    .compute_snarked_ledger_hashes(&snarked_ledger_hash);

                if let Err(_error) = compute_hashes_result {
                    // TODO(tizoc): log this error
                }

                store.dispatch(
                    TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsAccepted {
                        address: address.clone(),
                        count: accounts.len() as u64,
                        sender: *sender,
                    },
                );
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsAccepted { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsRejected { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressPending { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::Success => {}
        }
    }
}

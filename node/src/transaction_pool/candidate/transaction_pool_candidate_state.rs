#![allow(clippy::unit_arg)]

use std::collections::{BTreeMap, BTreeSet};

use mina_p2p_messages::v2;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::core::transaction::{
    Transaction, TransactionHash, TransactionInfo, TransactionWithHash,
};
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;

static EMPTY_PEER_TX_CANDIDATES: BTreeMap<TransactionHash, TransactionPoolCandidateState> =
    BTreeMap::new();

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransactionPoolCandidatesState {
    by_peer: BTreeMap<PeerId, BTreeMap<TransactionHash, TransactionPoolCandidateState>>,
    by_hash: BTreeMap<TransactionHash, BTreeSet<PeerId>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPoolCandidateState {
    InfoReceived {
        time: Timestamp,
        info: TransactionInfo,
    },
    FetchPending {
        time: Timestamp,
        info: TransactionInfo,
        rpc_id: P2pRpcId,
    },
    Received {
        time: Timestamp,
        transaction: TransactionWithHash,
    },
    VerifyPending {
        time: Timestamp,
        transaction: TransactionWithHash,
        verify_id: (),
    },
    VerifyError {
        time: Timestamp,
        transaction: TransactionWithHash,
    },
    VerifySuccess {
        time: Timestamp,
        transaction: TransactionWithHash,
    },
}

impl TransactionPoolCandidatesState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transactions_count(&self) -> usize {
        self.by_hash.len()
    }

    pub fn peer_transaction_count(&self, peer_id: &PeerId) -> usize {
        self.by_peer.get(peer_id).map(|v| v.len()).unwrap_or(0)
    }

    pub fn contains(&self, hash: &TransactionHash) -> bool {
        self.by_hash.contains_key(hash)
    }

    pub fn get(
        &self,
        peer_id: PeerId,
        hash: &TransactionHash,
    ) -> Option<&TransactionPoolCandidateState> {
        self.by_peer.get(&peer_id)?.get(hash)
    }

    fn transactions_from_peer_or_empty(
        &self,
        peer_id: PeerId,
    ) -> &BTreeMap<TransactionHash, TransactionPoolCandidateState> {
        self.by_peer
            .get(&peer_id)
            .unwrap_or(&EMPTY_PEER_TX_CANDIDATES)
    }

    pub fn candidates_from_peer_iter(
        &self,
        peer_id: PeerId,
    ) -> impl Iterator<Item = (&TransactionHash, &TransactionPoolCandidateState)> {
        self.transactions_from_peer_or_empty(peer_id).iter()
    }

    pub fn candidates_from_peer_with_hashes<'a, I>(
        &'a self,
        peer_id: PeerId,
        transaction_hashes: I,
    ) -> impl Iterator<
        Item = (
            &'a TransactionHash,
            Option<&'a TransactionPoolCandidateState>,
        ),
    >
    where
        I: IntoIterator<Item = &'a TransactionHash>,
    {
        let transactions = self.transactions_from_peer_or_empty(peer_id);
        transaction_hashes
            .into_iter()
            .map(|hash| (hash, transactions.get(hash)))
    }

    pub fn info_received(&mut self, time: Timestamp, peer_id: PeerId, info: TransactionInfo) {
        self.by_hash
            .entry(info.hash.clone())
            .or_default()
            .insert(peer_id);

        let hash = info.hash.clone();
        let state = TransactionPoolCandidateState::InfoReceived { time, info };
        self.by_peer.entry(peer_id).or_default().insert(hash, state);
    }

    pub fn peers_next_transactions_to_fetch<I, F>(
        &self,
        peers: I,
        get_order: F,
    ) -> Vec<(PeerId, TransactionHash)>
    where
        I: IntoIterator<Item = PeerId>,
        F: Copy + Fn(&TransactionHash) -> usize,
    {
        let mut needs_fetching = peers
            .into_iter()
            .filter_map(|peer_id| Some((peer_id, self.by_peer.get(&peer_id)?)))
            .flat_map(|(peer_id, transactions)| {
                transactions
                    .iter()
                    .filter(|(_, state)| {
                        matches!(state, TransactionPoolCandidateState::InfoReceived { .. })
                    })
                    .map(move |(hash, state)| (get_order(hash), state.fee(), peer_id, hash))
            })
            .collect::<Vec<_>>();
        needs_fetching
            .sort_by(|(ord1, fee1, ..), (ord2, fee2, ..)| ord1.cmp(ord2).then(fee1.cmp(fee2)));

        needs_fetching
            .into_iter()
            .scan(None, |last_ord, (ord, _, peer_id, hash)| {
                if *last_ord == Some(ord) {
                    return Some(None);
                }
                *last_ord = Some(ord);
                Some(Some((peer_id, hash.clone())))
            })
            .flatten()
            .collect()
    }

    pub fn work_fetch_pending(
        &mut self,
        time: Timestamp,
        peer_id: &PeerId,
        hash: &TransactionHash,
        rpc_id: P2pRpcId,
    ) {
        if let Some(state) = self
            .by_peer
            .get_mut(peer_id)
            .and_then(|transactions| transactions.get_mut(hash))
        {
            if let TransactionPoolCandidateState::InfoReceived { info, .. } = state {
                *state = TransactionPoolCandidateState::FetchPending {
                    time,
                    info: info.clone(),
                    rpc_id,
                };
            }
        }
    }

    pub fn work_received(
        &mut self,
        time: Timestamp,
        peer_id: PeerId,
        transaction: TransactionWithHash,
    ) {
        let hash = transaction.hash().clone();
        self.by_hash
            .entry(hash.clone())
            .or_default()
            .insert(peer_id);

        let state = TransactionPoolCandidateState::Received { time, transaction };
        self.by_peer.entry(peer_id).or_default().insert(hash, state);
    }

    pub fn get_batch_to_verify(&self) -> Option<(PeerId, Vec<TransactionWithHash>)> {
        for hash in self.by_hash.keys() {
            if let Some(res) = None.or_else(|| {
                for peer_id in self.by_hash.get(hash)? {
                    let peer_transactions = self.by_peer.get(peer_id)?;
                    if peer_transactions.get(hash)?.transaction().is_some() {
                        let transactions = peer_transactions
                            .iter()
                            .filter_map(|(_, v)| match v {
                                TransactionPoolCandidateState::Received { transaction, .. } => {
                                    Some(transaction)
                                }
                                _ => None,
                            })
                            .cloned()
                            .collect();
                        return Some((*peer_id, transactions));
                    }
                }
                None
            }) {
                return Some(res);
            }
        }
        None
    }

    pub fn verify_pending(
        &mut self,
        time: Timestamp,
        peer_id: &PeerId,
        verify_id: (),
        transaction_hashes: &[TransactionHash],
    ) {
        let Some(peer_transactions) = self.by_peer.get_mut(peer_id) else {
            return;
        };

        for hash in transaction_hashes {
            if let Some(job_state) = peer_transactions.get_mut(hash) {
                if let TransactionPoolCandidateState::Received { transaction, .. } = job_state {
                    *job_state = TransactionPoolCandidateState::VerifyPending {
                        time,
                        transaction: transaction.clone(),
                        verify_id,
                    };
                }
            }
        }
    }

    pub fn verify_result(
        &mut self,
        _time: Timestamp,
        peer_id: &PeerId,
        verify_id: (),
        _result: Result<(), ()>,
    ) {
        if let Some(peer_transactions) = self.by_peer.get_mut(peer_id) {
            let txs_to_remove = peer_transactions
                .iter()
                .filter(|(_, job_state)| job_state.pending_verify_id() == Some(verify_id))
                .map(|(hash, _)| hash.clone())
                .collect::<Vec<_>>();

            for hash in txs_to_remove {
                self.transaction_remove(&hash);
            }
        }
    }

    pub fn peer_remove(&mut self, peer_id: PeerId) {
        if let Some(works) = self.by_peer.remove(&peer_id) {
            for hash in works.into_keys() {
                if let Some(peers) = self.by_hash.get_mut(&hash) {
                    peers.remove(&peer_id);
                    if peers.is_empty() {
                        self.by_hash.remove(&hash);
                    }
                }
            }
        }
    }

    fn transaction_remove(&mut self, hash: &TransactionHash) {
        if let Some(peers) = self.by_hash.remove(hash) {
            for peer_id in peers {
                if let Some(txs) = self.by_peer.get_mut(&peer_id) {
                    txs.remove(hash);
                }
            }
        }
    }

    pub fn remove_inferior_transactions(&mut self, transaction: &Transaction) {
        // TODO(binier)
        match transaction.hash() {
            Err(err) => {
                openmina_core::bug_condition!("tx hashing failed: {err}");
            }
            Ok(hash) => self.transaction_remove(&hash),
        };
    }

    pub fn retain<F1, F2>(&mut self, mut predicate: F1)
    where
        F1: FnMut(&TransactionHash) -> F2,
        F2: FnMut(&TransactionPoolCandidateState) -> bool,
    {
        let by_peer = &mut self.by_peer;
        self.by_hash.retain(|hash, peers| {
            let mut predicate = predicate(hash);
            peers.retain(|peer_id| {
                if let Some(peer_works) = by_peer.get_mut(peer_id) {
                    match peer_works.get(hash) {
                        Some(s) if predicate(s) => true,
                        Some(_) => {
                            peer_works.remove(hash);
                            false
                        }
                        None => false,
                    }
                } else {
                    false
                }
            });
            !peers.is_empty()
        })
    }
}

impl TransactionPoolCandidateState {
    pub fn fee(&self) -> u64 {
        match self {
            Self::InfoReceived { info, .. } | Self::FetchPending { info, .. } => info.fee,
            Self::Received { transaction, .. }
            | Self::VerifyPending { transaction, .. }
            | Self::VerifyError { transaction, .. }
            | Self::VerifySuccess { transaction, .. } => match transaction.body() {
                v2::MinaBaseUserCommandStableV2::SignedCommand(v) => v.payload.common.fee.as_u64(),
                v2::MinaBaseUserCommandStableV2::ZkappCommand(v) => v.fee_payer.body.fee.as_u64(),
            },
        }
    }

    pub fn transaction(&self) -> Option<&TransactionWithHash> {
        match self {
            Self::InfoReceived { .. } => None,
            Self::FetchPending { .. } => None,
            Self::Received { transaction, .. } => Some(transaction),
            Self::VerifyPending { transaction, .. } => Some(transaction),
            Self::VerifyError { transaction, .. } => Some(transaction),
            Self::VerifySuccess { transaction, .. } => Some(transaction),
        }
    }

    pub fn pending_verify_id(&self) -> Option<()> {
        match self {
            Self::VerifyPending { verify_id, .. } => Some(*verify_id),
            _ => None,
        }
    }
}

use super::zkapp_command::{self, AccountUpdate, CallForest, Tree};
use crate::hash::{hash_with_kimchi, HashParam};
use ark_ff::Zero;
use mina_curves::pasta::Fp;
use mina_hasher::{Hashable, ROInput};
use mina_signer::NetworkId;

#[derive(Copy, Clone, Debug, derive_more::Deref, derive_more::From)]
pub struct TransactionCommitment(pub Fp);

impl TransactionCommitment {
    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L1365>
    pub fn create(account_updates_hash: Fp) -> Self {
        Self(account_updates_hash)
    }

    /// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L1368>
    pub fn create_complete(&self, memo_hash: Fp, fee_payer_hash: Fp) -> Self {
        Self(hash_with_kimchi(
            HashParam::AccountUpdateCons,
            &[memo_hash, fee_payer_hash, self.0],
        ))
    }

    pub fn empty() -> Self {
        Self(Fp::zero())
    }
}

impl Hashable for TransactionCommitment {
    type D = NetworkId;

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new();
        roi = roi.append_field(self.0);
        roi
    }

    fn domain_string(network_id: NetworkId) -> Option<String> {
        let param = match network_id {
            NetworkId::MAINNET => mina_core::HashParam::SignatureMainnet,
            NetworkId::TESTNET => mina_core::HashParam::CodaSignature,
        };
        let s: &str = param.into();
        Some(s.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct ZkappStatement {
    pub account_update: TransactionCommitment,
    pub calls: TransactionCommitment,
}

impl ZkappStatement {
    pub fn to_field_elements(&self) -> Vec<Fp> {
        let Self {
            account_update,
            calls,
        } = self;

        vec![**account_update, **calls]
    }

    pub fn of_tree<AccUpdate: Clone + zkapp_command::AccountUpdateRef>(
        tree: &Tree<AccUpdate>,
    ) -> Self {
        let Tree {
            account_update: _,
            account_update_digest,
            calls,
        } = tree;

        Self {
            account_update: TransactionCommitment(account_update_digest.get().unwrap()),
            calls: TransactionCommitment(calls.hash()),
        }
    }

    pub fn zkapp_statements_of_forest_prime<Data: Clone>(
        forest: CallForest<(AccountUpdate, Data)>,
    ) -> CallForest<(AccountUpdate, (Data, Self))> {
        forest.map_with_trees_to(|(account_update, data), tree| {
            (account_update.clone(), (data.clone(), Self::of_tree(tree)))
        })
    }

    fn zkapp_statements_of_forest(
        forest: CallForest<AccountUpdate>,
    ) -> CallForest<(AccountUpdate, Self)> {
        forest
            .map_with_trees_to(|account_update, tree| (account_update.clone(), Self::of_tree(tree)))
    }
}

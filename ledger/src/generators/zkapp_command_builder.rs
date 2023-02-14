use std::collections::HashMap;

use ark_ff::One;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_signer::{Keypair, Signature};

use crate::{
    hash_with_kimchi,
    scan_state::transaction_logic::zkapp_command::{AccountUpdate, Control, ZkAppCommand},
    staged_ledger::pre_diff_info::HashableCompressedPubKey,
};

#[derive(Clone, Debug)]
pub struct TransactionCommitment(pub Fp);

impl TransactionCommitment {
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L1365
    fn create(account_updates_hash: Fp) -> Self {
        Self(account_updates_hash)
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L1368
    fn create_complete(&self, memo_hash: Fp, fee_payer_hash: Fp) -> Self {
        Self(hash_with_kimchi(
            "MinaAcctUpdateCons",
            &[memo_hash, fee_payer_hash, self.0],
        ))
    }
}

pub fn get_transaction_commitments(
    zkapp_command: &ZkAppCommand,
) -> (TransactionCommitment, TransactionCommitment) {
    let memo_hash = zkapp_command.memo.hash();
    let account_updates_hash = zkapp_command.account_updates_hash();
    let fee_payer_hash = AccountUpdate::of_fee_payer(zkapp_command.fee_payer.clone()).digest();

    let txn_commitment = TransactionCommitment::create(account_updates_hash);
    let full_txn_commitment = txn_commitment.create_complete(memo_hash, fee_payer_hash);

    (txn_commitment, full_txn_commitment)
}

/// replace dummy signatures, proofs with valid ones for fee payer, other zkapp_command
/// [keymap] maps compressed public keys to private keys
///
/// https://github.com/MinaProtocol/mina/blob/f7f6700332bdfca77d9f3303e9cf3bc25f997e09/src/lib/zkapp_command_builder/zkapp_command_builder.ml#L94
pub fn replace_authorizations(
    prover: Option<()>, // TODO: We don't support that yet
    keymap: &HashMap<HashableCompressedPubKey, Keypair>,
    zkapp_command: &mut ZkAppCommand,
) {
    let (txn_commitment, full_txn_commitment) = get_transaction_commitments(zkapp_command);

    let sign_for_account_update = |use_full_commitment: bool, _kp: &Keypair| {
        let _commitment = if use_full_commitment {
            full_txn_commitment.clone()
        } else {
            txn_commitment.clone()
        };

        // TODO: Really sign the zkapp
        Signature {
            rx: Fp::one(),
            s: Fq::one(),
        }
    };

    let fee_payer_kp = keymap
        .get(&HashableCompressedPubKey(
            zkapp_command.fee_payer.body.public_key.clone(),
        ))
        .unwrap();

    let fee_payer_signature = sign_for_account_update(true, fee_payer_kp);

    zkapp_command.fee_payer.authorization = fee_payer_signature;

    let account_updates_with_valid_signatures =
        zkapp_command.account_updates.map_to(|account_update| {
            let AccountUpdate {
                body,
                authorization,
            } = account_update;

            let authorization_with_valid_signature = match authorization {
                Control::Signature(_dummy) => {
                    let pk = &body.public_key;
                    let kp = keymap
                        .get(&HashableCompressedPubKey(pk.clone()))
                        .expect("Could not find private key for public key in keymap");

                    let use_full_commitment = body.use_full_commitment;
                    let signature = sign_for_account_update(use_full_commitment, kp);
                    Control::Signature(signature)
                }
                Control::Proof(_) => match prover {
                    None => authorization.clone(),
                    Some(_prover) => todo!(), // TODO
                },
                Control::NoneGiven => authorization.clone(),
            };

            AccountUpdate {
                authorization: authorization_with_valid_signature,
                ..account_update.clone()
            }
        });

    zkapp_command.account_updates = account_updates_with_valid_signatures;
}

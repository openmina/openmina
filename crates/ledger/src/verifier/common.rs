use crate::{
    decompress_pk,
    scan_state::transaction_logic::{
        transaction_union_payload::TransactionUnionPayload,
        valid, verifiable,
        zkapp_command::{self, valid::of_verifiable, AccountUpdate},
        zkapp_statement::{TransactionCommitment, ZkappStatement},
        TransactionStatus, WithStatus,
    },
    VerificationKey,
};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use mina_p2p_messages::v2::PicklesProofProofsVerifiedMaxStableV2;
use mina_signer::{CompressedPubKey, PubKey, Signature};
use poseidon::hash::hash_with_kimchi;
use std::sync::Arc;

#[derive(Debug)]
pub enum CheckResult {
    Valid(valid::UserCommand),
    ValidAssuming(
        (
            valid::UserCommand,
            Vec<(
                VerificationKey,
                ZkappStatement,
                Arc<PicklesProofProofsVerifiedMaxStableV2>,
            )>,
        ),
    ),
    InvalidKeys(Vec<CompressedPubKey>),
    InvalidSignature(Vec<CompressedPubKey>),
    InvalidProof(String),
    MissingVerificationKey(Vec<CompressedPubKey>),
    UnexpectedVerificationKey(Vec<CompressedPubKey>),
    MismatchedAuthorizationKind(Vec<CompressedPubKey>),
}

/// <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/verifier/common.ml#L29>
pub fn check(cmd: WithStatus<verifiable::UserCommand>) -> CheckResult {
    use verifiable::UserCommand::{SignedCommand, ZkAppCommand};

    match cmd.data {
        SignedCommand(cmd) => {
            if !cmd.check_valid_keys() {
                let public_keys = cmd.public_keys().into_iter().cloned().collect();
                return CheckResult::InvalidKeys(public_keys);
            }
            match verifiable::check_only_for_signature(cmd) {
                Ok(cmd) => CheckResult::Valid(cmd),
                Err(cmd) => {
                    CheckResult::InvalidSignature(cmd.public_keys().into_iter().cloned().collect())
                }
            }
        }
        ZkAppCommand(zkapp_command_with_vk) => {
            let zkapp_command::verifiable::ZkAppCommand {
                fee_payer,
                account_updates,
                memo,
            } = &*zkapp_command_with_vk;

            let account_updates_hash = account_updates.hash();
            let tx_commitment = TransactionCommitment::create(account_updates_hash);

            let memo_hash = memo.hash();
            let fee_payer_hash = AccountUpdate::of_fee_payer(fee_payer.clone()).digest();
            let full_tx_commitment = tx_commitment.create_complete(memo_hash, fee_payer_hash);

            let Some(pk) = decompress_pk(&fee_payer.body.public_key) else {
                return CheckResult::InvalidKeys(vec![fee_payer.body.public_key.clone()]);
            };

            if !verify_signature(&fee_payer.authorization, &pk, &full_tx_commitment) {
                return CheckResult::InvalidSignature(vec![pk.into_compressed()]);
            }

            let zkapp_command_with_hashes_list =
                ZkappStatement::zkapp_statements_of_forest_prime(account_updates.clone())
                    .to_zkapp_command_with_hashes_list();

            let mut valid_assuming = Vec::with_capacity(16);
            for ((p, (vk_opt, stmt)), _at_account_update) in zkapp_command_with_hashes_list {
                let commitment = if p.body.use_full_commitment {
                    full_tx_commitment
                } else {
                    tx_commitment
                };

                use zkapp_command::{AuthorizationKind as AK, Control as C};
                match (&p.authorization, &p.body.authorization_kind) {
                    (C::Signature(s), AK::Signature) => {
                        let pk = decompress_pk(&p.body.public_key).unwrap();
                        if !verify_signature(s, &pk, &commitment) {
                            return CheckResult::InvalidSignature(vec![pk.into_compressed()]);
                        }
                        continue;
                    }
                    (C::NoneGiven, AK::NoneGiven) => {
                        continue;
                    }
                    (C::Proof(pi), AK::Proof(vk_hash)) => {
                        if let TransactionStatus::Failed(_) = cmd.status {
                            // Don't verify the proof if it has failed.
                            continue;
                        }
                        let Some(vk) = vk_opt else {
                            return CheckResult::MissingVerificationKey(vec![
                                p.account_id().public_key,
                            ]);
                        };
                        // check that vk expected for proof is the one being used
                        if vk_hash != &vk.hash() {
                            return CheckResult::UnexpectedVerificationKey(vec![
                                p.account_id().public_key,
                            ]);
                        }
                        valid_assuming.push((vk.vk().clone(), stmt, pi.clone()));
                    }
                    _ => {
                        return CheckResult::MismatchedAuthorizationKind(vec![
                            p.account_id().public_key,
                        ]);
                    }
                }
            }

            let v: valid::UserCommand = {
                // Verification keys should be present if it reaches here
                let zkapp = of_verifiable(*zkapp_command_with_vk);
                valid::UserCommand::ZkAppCommand(Box::new(zkapp))
            };

            if valid_assuming.is_empty() {
                CheckResult::Valid(v)
            } else {
                CheckResult::ValidAssuming((v, valid_assuming))
            }
        }
    }
}

/// Verify zkapp signature/statement with new style (chunked inputs)
fn verify_signature(signature: &Signature, pubkey: &PubKey, msg: &TransactionCommitment) -> bool {
    use ark_ff::{BigInteger, Zero};
    use core::ops::{Mul, Neg};
    use mina_curves::pasta::{Fq, Pallas};
    use mina_signer::CurvePoint;

    let Pallas { x, y, .. } = pubkey.point();
    let Signature { rx, s } = signature;

    let signature_prefix = mina_core::NetworkConfig::global().signature_prefix;
    let hash = hash_with_kimchi(signature_prefix, &[**msg, *x, *y, *rx]);
    let hash: Fq = Fq::from(hash.into_bigint()); // Never fail, `Fq` is larger than `Fp`

    let sv: CurvePoint = CurvePoint::generator().mul(*s).into_affine();
    // Perform addition and infinity check in projective coordinates for performance
    let rv = pubkey.point().mul(hash).neg() + sv;
    if rv.is_zero() {
        return false;
    }
    let rv = rv.into_affine();
    rv.y.into_bigint().is_even() && rv.x == *rx
}

/// Verify signature with legacy style
pub fn legacy_verify_signature(
    signature: &Signature,
    pubkey: &PubKey,
    msg: &TransactionUnionPayload,
) -> bool {
    use ::poseidon::hash::legacy;
    use ark_ff::{BigInteger, Zero};
    use core::ops::{Mul, Neg};
    use mina_curves::pasta::{Fq, Pallas};
    use mina_signer::CurvePoint;

    let Pallas { x, y, .. } = pubkey.point();
    let Signature { rx, s } = signature;

    let signature_prefix = mina_core::NetworkConfig::global().legacy_signature_prefix;

    let mut inputs = msg.to_input_legacy();
    inputs.append_field(*x);
    inputs.append_field(*y);
    inputs.append_field(*rx);

    let hash = legacy::hash_with_kimchi(signature_prefix, &inputs.to_fields());
    let hash: Fq = Fq::from(hash.into_bigint()); // Never fail, `Fq` is larger than `Fp`

    let sv: CurvePoint = CurvePoint::generator().mul(*s).into_affine();
    // Perform addition and infinity check in projective coordinates for performance
    let rv = pubkey.point().mul(hash).neg() + sv;
    if rv.is_zero() {
        return false;
    }
    let rv = rv.into_affine();
    rv.y.into_bigint().is_even() && rv.x == *rx
}

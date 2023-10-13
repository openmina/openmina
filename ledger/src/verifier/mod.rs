use std::sync::Arc;

use crate::{
    proofs::{verification, verifier_index::get_verifier_index, VerifierIndex, VerifierSRS},
    scan_state::{
        scan_state::transaction_snark::{
            LedgerProof, LedgerProofWithSokMessage, SokMessage, TransactionSnark,
        },
        transaction_logic::{valid, verifiable, zkapp_statement::ZkappStatement, WithStatus},
    },
    staged_ledger::staged_ledger::SkipVerification,
    VerificationKey,
};

use self::common::CheckResult;

#[derive(Debug, Clone)]
pub struct Verifier;

use mina_p2p_messages::v2::{
    PicklesProofProofsVerified2ReprStableV2, PicklesProofProofsVerifiedMaxStableV2,
};
use mina_signer::CompressedPubKey;
use once_cell::sync::Lazy;

// TODO: Move this into `Verifier` struct above
static VERIFIER_INDEX: Lazy<Arc<VerifierIndex>> = Lazy::new(|| {
    use crate::proofs::verifier_index::VerifierKind;
    Arc::new(get_verifier_index(VerifierKind::Transaction))
});

// TODO: Move this into `Verifier` struct above
pub static SRS: Lazy<Arc<VerifierSRS>> =
    Lazy::new(|| std::sync::Arc::new(crate::proofs::accumulator_check::get_srs()));

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_snark/transaction_snark.ml#L3492
fn verify(ts: Vec<(LedgerProof, SokMessage)>) -> Result<(), String> {
    if ts.iter().all(|(proof, msg)| {
        let LedgerProof(TransactionSnark { statement, .. }) = proof;
        statement.sok_digest == msg.digest()
    }) {
        let verifier_index = VERIFIER_INDEX.as_ref();
        let srs = SRS.as_ref();

        // for (proof, msg) in ts {
        //     let LedgerProof(TransactionSnark {
        //         statement,
        //         proof: p,
        //     }) = &proof;
        //     let (stmt, p) = (statement, &**p);
        //     if !crate::proofs::verification::verify_transaction([(stmt, p)], verifier_index) {
        //         let a: mina_p2p_messages::v2::LedgerProofProdStableV2 = (&proof).into();
        //         let b: mina_p2p_messages::v2::MinaBaseSokMessageStableV1 = (&msg).into();
        //         let mut file = std::fs::File::create("ledger_proof2.bin").unwrap();
        //         binprot::BinProtWrite::binprot_write(&a, &mut file).unwrap();
        //         file.sync_all().unwrap();
        //         let mut file = std::fs::File::create("sok_msg2.bin").unwrap();
        //         binprot::BinProtWrite::binprot_write(&b, &mut file).unwrap();
        //         file.sync_all().unwrap();
        //         panic!();
        //     }
        // }

        let proofs = ts.iter().map(|(proof, _)| {
            let LedgerProof(TransactionSnark { statement, proof }) = proof;
            (statement, &**proof)
        });

        if !crate::proofs::verification::verify_transaction(proofs, verifier_index, srs) {
            return Err("Transaction_snark.verify: verification failed".into());
        }
        Ok(())
    } else {
        Err("Transaction_snark.verify: Mismatched sok_message".into())
    }
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/verifier/dummy.ml#L59C1-L75C81
#[cfg(test)]
fn verify_digest_only(ts: Vec<(LedgerProof, SokMessage)>) -> Result<(), String> {
    use crate::scan_state::scan_state::transaction_snark::SokDigest;

    if ts.iter().all(|(proof, msg)| {
        let LedgerProof(TransactionSnark { statement, .. }) = proof;
        statement.sok_digest == SokDigest::default() || statement.sok_digest == msg.digest()
    }) {
        Ok(())
    } else {
        Err("Transaction_snark.verify: Mismatched sok_message".into())
    }
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/verifier/verifier_intf.ml#L10C1-L36C29
pub enum VerifyCommandsResult {
    Valid(valid::UserCommand),
    ValidAssuming(
        Vec<(
            VerificationKey,
            ZkappStatement,
            Arc<PicklesProofProofsVerifiedMaxStableV2>,
        )>,
    ),
    InvalidKeys(Vec<CompressedPubKey>),
    InvalidSignature(Vec<CompressedPubKey>),
    InvalidProof(String),
    MissingVerificationKey(Vec<CompressedPubKey>),
    UnexpectedVerificationKey(Vec<CompressedPubKey>),
    MismatchedVerificationKey(Vec<CompressedPubKey>),
    MismatchedAuthorizationKind(Vec<CompressedPubKey>),
}

impl Verifier {
    pub fn verify(&self, _proofs: &[LedgerProofWithSokMessage]) -> Result<Result<(), ()>, String> {
        // Implement verification later
        //
        // https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/pickles/pickles.ml#L1122
        // https://viable-systems.slack.com/archives/D01SVA87PQC/p1671715846448749
        Ok(Ok(()))
    }

    /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/verifier/prod.ml#L138
    #[allow(unreachable_code)]
    pub fn verify_transaction_snarks(
        &self,
        ts: Vec<(LedgerProof, SokMessage)>,
    ) -> Result<(), String> {
        #[cfg(test)]
        return verify_digest_only(ts);

        verify(ts)
    }

    pub fn verify_commands(
        &self,
        cmds: Vec<WithStatus<verifiable::UserCommand>>,
        skip_verification: Option<SkipVerification>,
    ) -> Vec<VerifyCommandsResult> {
        let cs: Vec<_> = cmds.into_iter().map(common::check).collect();

        let mut to_verify = cs
            .iter()
            .filter_map(|c| match c {
                CheckResult::Valid(_) => None,
                CheckResult::ValidAssuming((_, xs)) => Some(xs),
                _ => None,
            })
            .flatten();

        let all_verified = if skip_verification.is_some() {
            true
        } else {
            let srs = SRS.as_ref();

            to_verify.all(|(vk, zkapp_statement, proof)| {
                let proof: PicklesProofProofsVerified2ReprStableV2 = (&**proof).into();
                verification::verify_zkapp(vk, zkapp_statement.clone(), &proof, srs)
            })
        };

        cs.into_iter()
            .map(|c| match c {
                CheckResult::Valid(c) => VerifyCommandsResult::Valid(c),
                CheckResult::ValidAssuming((c, xs)) => {
                    if all_verified {
                        VerifyCommandsResult::Valid(c)
                    } else {
                        VerifyCommandsResult::ValidAssuming(xs)
                    }
                }
                CheckResult::InvalidKeys(keys) => VerifyCommandsResult::InvalidKeys(keys),
                CheckResult::InvalidSignature(keys) => VerifyCommandsResult::InvalidSignature(keys),
                CheckResult::InvalidProof(s) => VerifyCommandsResult::InvalidProof(s),
                CheckResult::MissingVerificationKey(keys) => {
                    VerifyCommandsResult::MissingVerificationKey(keys)
                }
                CheckResult::UnexpectedVerificationKey(keys) => {
                    VerifyCommandsResult::UnexpectedVerificationKey(keys)
                }
                CheckResult::MismatchedAuthorizationKind(keys) => {
                    VerifyCommandsResult::MismatchedAuthorizationKind(keys)
                }
            })
            .collect()
    }
}

#[derive(Debug, derive_more::From)]
pub enum VerifierError {
    CheckError(CheckResult),
    VerificationFailed(String),
}

pub mod common {
    use std::sync::Arc;

    use mina_p2p_messages::v2::PicklesProofProofsVerifiedMaxStableV2;
    use mina_signer::{CompressedPubKey, PubKey, Signature};

    use crate::{
        decompress_pk, hash_with_kimchi,
        scan_state::transaction_logic::{
            valid, verifiable,
            zkapp_command::{self, valid::of_verifiable, AccountUpdate},
            zkapp_statement::{TransactionCommitment, ZkappStatement},
            TransactionStatus, WithStatus,
        },
        VerificationKey,
    };

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

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/verifier/common.ml#L29
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
                    Err(cmd) => CheckResult::InvalidSignature(
                        cmd.public_keys().into_iter().cloned().collect(),
                    ),
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

                    use zkapp_command::AuthorizationKind as AK;
                    use zkapp_command::Control as C;
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
                            if vk_hash != &vk.hash {
                                return CheckResult::UnexpectedVerificationKey(vec![
                                    p.account_id().public_key,
                                ]);
                            }
                            valid_assuming.push((vk.data, stmt, pi.clone()));
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

    /// Verify signature with new style (chunked inputs)
    /// `mina_signer::verify` is using old one
    fn verify_signature(
        signature: &Signature,
        pubkey: &PubKey,
        msg: &TransactionCommitment,
    ) -> bool {
        use ark_ec::{AffineCurve, ProjectiveCurve};
        use ark_ff::{BigInteger, PrimeField, Zero};
        use mina_curves::pasta::Fq;
        use mina_curves::pasta::Pallas;
        use mina_signer::CurvePoint;
        use std::ops::Neg;

        let Pallas { x, y, .. } = pubkey.point();
        let Signature { rx, s } = signature;

        // TODO: Change when it become mainnet ?
        // let signature_testnet = create "CodaSignature"
        // let signature_mainnet = create "MinaSignatureMainnet"

        let hash = hash_with_kimchi("CodaSignature", &[**msg, *x, *y, *rx]);
        let hash: Fq = Fq::from(hash.into_repr());

        let sv: CurvePoint = CurvePoint::prime_subgroup_generator().mul(*s).into_affine();
        // Perform addition and infinity check in projective coordinates for performance
        let rv = pubkey.point().mul(hash).neg().add_mixed(&sv);
        if rv.is_zero() {
            return false;
        }
        let rv = rv.into_affine();
        rv.y.into_repr().is_even() && rv.x == *rx
    }
}

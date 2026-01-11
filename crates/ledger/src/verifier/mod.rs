use self::common::CheckResult;
use crate::{
    proofs::{
        self, field::FieldWitness, verification, verifiers::TransactionVerifier, VerifierIndex,
    },
    scan_state::{
        scan_state::transaction_snark::{
            LedgerProof, LedgerProofWithSokMessage, SokMessage, TransactionSnark,
        },
        transaction_logic::{valid, verifiable, zkapp_statement::ZkappStatement, WithStatus},
    },
    staged_ledger::staged_ledger::SkipVerification,
    VerificationKey,
};
use mina_curves::pasta::{Fp, Fq};
use mina_p2p_messages::v2::{
    PicklesProofProofsVerified2ReprStableV2, PicklesProofProofsVerifiedMaxStableV2,
};
use mina_signer::CompressedPubKey;
use once_cell::sync::Lazy;
use poly_commitment::{ipa::SRS, SRS as _};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Verifier;

// TODO: Move this into `Verifier` struct above
pub static VERIFIER_INDEX: Lazy<Arc<VerifierIndex<Fq>>> = Lazy::new(|| {
    TransactionVerifier::get()
        .expect("verifier index not initialized")
        .into()
});

/// Returns the Structured Reference String (SRS) for proof verification.
/// Lazily created and cached globally. Immutable version for verifiers.
///
/// TODO: Use directly from proof-systems (<https://github.com/o1-labs/mina-rust/issues/1749>)
pub fn get_srs<F: FieldWitness>() -> Arc<SRS<F::OtherCurve>> {
    cache! {
        Arc<SRS<F::OtherCurve>>,
        {
            let srs = SRS::<F::OtherCurve>::create(<F as proofs::field::FieldWitness>::Scalar::SRS_DEPTH);
            Arc::new(srs)
        }
    }
}

/// Returns the SRS on the other curve (Mutex-wrapped version for prover).
///
/// TODO: Use directly from proof-systems (<https://github.com/o1-labs/mina-rust/issues/1749>)
pub fn get_srs_mut<F: FieldWitness>() -> Arc<Mutex<SRS<F::OtherCurve>>> {
    cache! {
        Arc<Mutex<SRS<F::OtherCurve>>>,
        {
            let srs = SRS::<F::OtherCurve>::create(<F as proofs::field::FieldWitness>::Scalar::SRS_DEPTH);
            Arc::new(Mutex::new(srs))
        }
    }
}

/// <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_snark/transaction_snark.ml#L3492>
fn verify(ts: Vec<(LedgerProof, SokMessage)>) -> Result<(), String> {
    let srs = get_srs::<Fp>();

    if ts.iter().all(|(proof, msg)| {
        let LedgerProof(TransactionSnark { statement, .. }) = proof;
        statement.sok_digest == msg.digest()
    }) {
        let verifier_index = VERIFIER_INDEX.as_ref();

        let proofs = ts.iter().map(|(proof, _)| {
            let LedgerProof(TransactionSnark { statement, proof }) = proof;
            (statement, &**proof)
        });

        if !crate::proofs::verification::verify_transaction(proofs, verifier_index, &srs) {
            return Err("Transaction_snark.verify: verification failed".into());
        }
        Ok(())
    } else {
        Err("Transaction_snark.verify: Mismatched sok_message".into())
    }
}

/// <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/verifier/dummy.ml#L59C1-L75C81>
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

/// <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/verifier/verifier_intf.ml#L10C1-L36C29>
pub type VerifyCommandsResult = Result<valid::UserCommand, VerifierError>;

#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    // TODO(adonagy): print something here as well?
    #[error("Batch verification failed")]
    ValidAssuming(
        Vec<(
            VerificationKey,
            ZkappStatement,
            Arc<PicklesProofProofsVerifiedMaxStableV2>,
        )>,
    ),
    #[error("Invalid keys: {0:?}")]
    InvalidKeys(Vec<CompressedPubKey>),
    #[error("Invalid signature: {0:?}")]
    InvalidSignature(Vec<CompressedPubKey>),
    #[error("Invalid proof: {0}")]
    InvalidProof(String),
    #[error("Missing verification key: {0:?}")]
    MissingVerificationKey(Vec<CompressedPubKey>),
    #[error("Unexpected verification key: {0:?}")]
    UnexpectedVerificationKey(Vec<CompressedPubKey>),
    #[error("Mismatched verification key: {0:?}")]
    MismatchedVerificationKey(Vec<CompressedPubKey>),
    #[error("Authorization kind does not match the authorization - Keys {0:?}")]
    MismatchedAuthorizationKind(Vec<CompressedPubKey>),
}

impl Verifier {
    pub fn verify(
        &self,
        _proofs: &[Arc<LedgerProofWithSokMessage>],
    ) -> Result<Result<(), ()>, String> {
        // Implement verification later
        //
        // <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/pickles/pickles.ml#L1122>
        // <https://viable-systems.slack.com/archives/D01SVA87PQC/p1671715846448749>
        Ok(Ok(()))
    }

    /// <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/verifier/prod.ml#L138>
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
            let srs = get_srs::<Fp>();

            to_verify.all(|(vk, zkapp_statement, proof)| {
                let proof: PicklesProofProofsVerified2ReprStableV2 = (&**proof).into();
                verification::verify_zkapp(vk, zkapp_statement, &proof, &srs)
            })
        };

        cs.into_iter()
            .map(|c| match c {
                CheckResult::Valid(c) => Ok(c),
                CheckResult::ValidAssuming((c, xs)) => {
                    if all_verified {
                        Ok(c)
                    } else {
                        Err(VerifierError::ValidAssuming(xs))
                    }
                }
                CheckResult::InvalidKeys(keys) => Err(VerifierError::InvalidKeys(keys)),
                CheckResult::InvalidSignature(keys) => Err(VerifierError::InvalidSignature(keys)),
                CheckResult::InvalidProof(s) => Err(VerifierError::InvalidProof(s)),
                CheckResult::MissingVerificationKey(keys) => {
                    Err(VerifierError::MissingVerificationKey(keys))
                }
                CheckResult::UnexpectedVerificationKey(keys) => {
                    Err(VerifierError::UnexpectedVerificationKey(keys))
                }
                CheckResult::MismatchedAuthorizationKind(keys) => {
                    Err(VerifierError::MismatchedAuthorizationKind(keys))
                }
            })
            .collect()
    }
}

pub mod common;

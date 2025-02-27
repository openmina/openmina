use std::sync::Arc;

use ark_ff::fields::arithmetic::InvalidBigInt;
use ledger::{
    scan_state::{
        scan_state::transaction_snark::{SokDigest, Statement},
        transaction_logic::WithStatus,
    },
    transaction_pool::{TransactionError, TransactionPoolErrors},
};
use mina_p2p_messages::v2;
use node::{
    core::{
        channels::mpsc,
        snark::{Snark, SnarkJobId},
        thread,
    },
    snark::{
        block_verify::{SnarkBlockVerifyError, SnarkBlockVerifyId, VerifiableBlockWithHash},
        work_verify::{SnarkWorkVerifyError, SnarkWorkVerifyId},
        BlockVerifier, SnarkEvent, TransactionVerifier, VerifierSRS,
    },
};
use rand::prelude::*;

use crate::NodeService;

use super::EventSender;

pub struct SnarkBlockVerifyArgs {
    pub req_id: SnarkBlockVerifyId,
    pub verifier_index: BlockVerifier,
    pub verifier_srs: Arc<VerifierSRS>,
    pub block: VerifiableBlockWithHash,
}

impl NodeService {
    pub fn snark_block_proof_verifier_spawn(
        event_sender: EventSender,
    ) -> mpsc::UnboundedSender<SnarkBlockVerifyArgs> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        thread::Builder::new()
            .name("block_proof_verifier".to_owned())
            .spawn(move || {
                while let Some(SnarkBlockVerifyArgs {
                    req_id,
                    verifier_index,
                    verifier_srs,
                    block,
                }) = rx.blocking_recv()
                {
                    eprintln!("verify({}) - start", block.hash_ref());
                    let header = block.header_ref();
                    let result = {
                        if !ledger::proofs::verification::verify_block(
                            header,
                            &verifier_index,
                            &verifier_srs,
                        ) {
                            Err(SnarkBlockVerifyError::VerificationFailed)
                        } else {
                            Ok(())
                        }
                    };
                    eprintln!("verify({}) - end", block.hash_ref());

                    let _ = event_sender.send(SnarkEvent::BlockVerify(req_id, result).into());
                }
            })
            .expect("failed to spawn block_proof_verifier thread");

        tx
    }
}

impl node::service::SnarkBlockVerifyService for NodeService {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: BlockVerifier,
        verifier_srs: Arc<VerifierSRS>,
        block: VerifiableBlockWithHash,
    ) {
        if self.replayer.is_some() {
            return;
        }
        let args = SnarkBlockVerifyArgs {
            req_id,
            verifier_index,
            verifier_srs,
            block,
        };
        let _ = self.snark_block_proof_verify.send(args);
    }
}

impl node::service::SnarkWorkVerifyService for NodeService {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: TransactionVerifier,
        verifier_srs: Arc<VerifierSRS>,
        work: Vec<Snark>,
    ) {
        if self.replayer.is_some() {
            return;
        }
        let tx = self.event_sender().clone();
        rayon::spawn_fifo(move || {
            let result = (|| {
                let conv = |proof: &v2::LedgerProofProdStableV2| -> Result<_, InvalidBigInt> {
                    Ok((
                        Statement::<SokDigest>::try_from(&proof.0.statement)?,
                        proof.proof.clone(),
                    ))
                };
                let Ok(works) = work
                    .into_iter()
                    .flat_map(|work| match &*work.proofs {
                        v2::TransactionSnarkWorkTStableV2Proofs::One(v) => {
                            [conv(v).map(Some), Ok(None)]
                        }
                        v2::TransactionSnarkWorkTStableV2Proofs::Two((v1, v2)) => {
                            [conv(v1).map(Some), conv(v2).map(Some)]
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
                else {
                    return Err(SnarkWorkVerifyError::VerificationFailed);
                };
                if !ledger::proofs::verification::verify_transaction(
                    works.iter().flatten().map(|(v1, v2)| (v1, v2)),
                    &verifier_index,
                    &verifier_srs,
                ) {
                    Err(SnarkWorkVerifyError::VerificationFailed)
                } else {
                    Ok(())
                }
            })();

            let _ = tx.send(SnarkEvent::WorkVerify(req_id, result).into());
        });
    }
}

impl node::service::SnarkUserCommandVerifyService for NodeService {
    fn verify_init(
        &mut self,
        req_id: node::snark::user_command_verify::SnarkUserCommandVerifyId,
        commands: Vec<WithStatus<ledger::scan_state::transaction_logic::verifiable::UserCommand>>,
    ) {
        if self.replayer.is_some() {
            return;
        }

        let tx = self.event_sender().clone();
        rayon::spawn_fifo(move || {
            let result = {
                let (verified, invalid): (Vec<_>, Vec<_>) = ledger::verifier::Verifier
                    .verify_commands(commands, None)
                    .into_iter()
                    .partition(Result::is_ok);

                let verified: Vec<_> = verified.into_iter().map(Result::unwrap).collect();
                let invalid: Vec<_> = invalid.into_iter().map(Result::unwrap_err).collect();

                if !invalid.is_empty() {
                    let transaction_pool_errors = invalid
                        .into_iter()
                        .map(TransactionError::Verifier)
                        .collect();
                    Err(TransactionPoolErrors::BatchedErrors(
                        transaction_pool_errors,
                    ))
                } else {
                    Ok(verified)
                }
            };

            let result = result.map_err(|err| err.to_string());

            let _ = tx.send(SnarkEvent::UserCommandVerify(req_id, result).into());
        });
    }
}

impl node::service::SnarkPoolService for NodeService {
    fn random_choose<'a>(
        &mut self,
        iter: impl Iterator<Item = &'a SnarkJobId>,
        n: usize,
    ) -> Vec<SnarkJobId> {
        iter.choose_multiple(&mut self.rng, n)
            .into_iter()
            .cloned()
            .collect()
    }
}

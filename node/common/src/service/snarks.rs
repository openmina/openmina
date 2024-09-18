use std::sync::Arc;

use ark_ff::fields::arithmetic::InvalidBigInt;
use ledger::scan_state::scan_state::transaction_snark::{SokDigest, Statement};
use mina_p2p_messages::v2;
use node::{
    core::{
        snark::{Snark, SnarkJobId},
        thread,
    },
    snark::{
        block_verify::{SnarkBlockVerifyError, SnarkBlockVerifyId, VerifiableBlockWithHash},
        work_verify::{SnarkWorkVerifyError, SnarkWorkVerifyId},
        SnarkEvent, VerifierIndex, VerifierSRS,
    },
};
use rand::prelude::*;

use crate::NodeService;

impl node::service::SnarkBlockVerifyService for NodeService {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        block: VerifiableBlockWithHash,
    ) {
        if self.replayer.is_some() {
            return;
        }
        let tx = self.event_sender().clone();
        thread::spawn(move || {
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

            let _ = tx.send(SnarkEvent::BlockVerify(req_id, result).into());
        });
    }
}

impl node::service::SnarkWorkVerifyService for NodeService {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: Arc<VerifierIndex>,
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
        _req_id: node::snark::user_command_verify::SnarkUserCommandVerifyId,
        _verifier_index: Arc<VerifierIndex>,
        _verifier_srs: Arc<VerifierSRS>,
        _commands: mina_p2p_messages::list::List<
            mina_p2p_messages::v2::MinaBaseUserCommandStableV2,
        >,
    ) {
        todo!()
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

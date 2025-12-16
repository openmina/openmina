use ledger::{
    proofs::{
        provers::{TransactionProver, ZkappProver},
        zkapp::ZkappParams,
    },
    scan_state::scan_state::transaction_snark::SokMessage,
};
use mina_p2p_messages::v2;
use mina_signer::CompressedPubKey;
use node::{
    core::channels::mpsc,
    event_source::ExternalSnarkWorkerEvent,
    external_snark_worker::{
        ExternalSnarkWorkerError, ExternalSnarkWorkerWorkError, SnarkWorkResult, SnarkWorkSpec,
        SnarkWorkSpecError,
    },
    snark::TransactionVerifier,
};

use crate::NodeService;

use super::EventSender;

pub struct SnarkWorker {
    cmd_sender: mpsc::UnboundedSender<Cmd>,
}

enum Cmd {
    Submit(Box<SnarkWorkSpec>),
    Cancel,
    Kill,
}

impl node::service::ExternalSnarkWorkerService for NodeService {
    fn start(
        &mut self,
        pub_key: v2::NonZeroCurvePoint,
        fee: v2::CurrencyFeeStableV1,
        work_verifier: TransactionVerifier,
    ) -> Result<(), ExternalSnarkWorkerError> {
        if self.replayer.is_some() {
            return Ok(());
        }
        let (cmd_sender, cmd_receiver) = mpsc::unbounded_channel();
        // TODO(binier): improve pub key conv
        let sok_message = SokMessage::create(
            (&fee).into(),
            CompressedPubKey::from_address(&pub_key.to_string()).unwrap(),
        );
        self.snark_worker = Some(SnarkWorker { cmd_sender });
        let event_sender = self.event_sender().clone();

        node::core::thread::Builder::new()
            .name("snark_worker".to_owned())
            .spawn(move || worker_thread(cmd_receiver, event_sender, sok_message, work_verifier))
            .map(|_| ())
            .map_err(|err| ExternalSnarkWorkerError::Error(err.to_string()))
    }

    fn kill(&mut self) -> Result<(), ExternalSnarkWorkerError> {
        if self.replayer.is_some() {
            return Ok(());
        }

        if self
            .snark_worker
            .as_ref()
            .and_then(|s| s.cmd_sender.send(Cmd::Kill).ok())
            .is_none()
        {
            return Err(ExternalSnarkWorkerError::NotRunning);
        }
        Ok(())
    }

    fn submit(&mut self, spec: SnarkWorkSpec) -> Result<(), ExternalSnarkWorkerError> {
        if self.replayer.is_some() {
            return Ok(());
        }

        if self
            .snark_worker
            .as_ref()
            .and_then(|s| s.cmd_sender.send(Cmd::Submit(spec.into())).ok())
            .is_none()
        {
            return Err(ExternalSnarkWorkerError::NotRunning);
        }
        Ok(())
    }

    fn cancel(&mut self) -> Result<(), ExternalSnarkWorkerError> {
        if self.replayer.is_some() {
            return Ok(());
        }

        // TODO(binier): for wasm threads, call terminate:
        // <https://developer.mozilla.org/en-US/docs/Web/API/Worker/terminate>
        if self
            .snark_worker
            .as_ref()
            .and_then(|s| s.cmd_sender.send(Cmd::Cancel).ok())
            .is_none()
        {
            return Err(ExternalSnarkWorkerError::NotRunning);
        }
        Ok(())
    }
}

fn worker_thread(
    mut cmd_receiver: mpsc::UnboundedReceiver<Cmd>,
    event_sender: EventSender,
    sok_message: SokMessage,
    work_verifier: TransactionVerifier,
) {
    let _ = event_sender.send(ExternalSnarkWorkerEvent::Started.into());
    let tx_prover = TransactionProver::make(Some(work_verifier.clone()));
    let zkapp_prover = ZkappProver::make(Some(work_verifier));
    while let Some(cmd) = cmd_receiver.blocking_recv() {
        match cmd {
            Cmd::Kill => {
                let _ = event_sender.send(ExternalSnarkWorkerEvent::Killed.into());
                return;
            }
            Cmd::Cancel => {
                // can't cancel as it's a blocking thread. Once this
                // is moved to another process, kill it.
                let _ = event_sender.send(ExternalSnarkWorkerEvent::WorkCancelled.into());
            }
            Cmd::Submit(spec) => {
                let event = match prove_spec(&tx_prover, &zkapp_prover, *spec, &sok_message) {
                    Err(err) => ExternalSnarkWorkerEvent::WorkError(err),
                    Ok(res) => ExternalSnarkWorkerEvent::WorkResult(res),
                };

                let _ = event_sender.send(event.into());
            }
        }
    }
}

fn prove_spec(
    tx_prover: &TransactionProver,
    zkapp_prover: &ZkappProver,
    spec: SnarkWorkSpec,
    sok_message: &SokMessage,
) -> Result<SnarkWorkResult, ExternalSnarkWorkerWorkError> {
    match spec {
        SnarkWorkSpec::One(single) => prove_single(tx_prover, zkapp_prover, single, sok_message)
            .map(v2::TransactionSnarkWorkTStableV2Proofs::One),
        SnarkWorkSpec::Two((one, two)) => Ok(v2::TransactionSnarkWorkTStableV2Proofs::Two((
            prove_single(tx_prover, zkapp_prover, one, sok_message)?,
            prove_single(tx_prover, zkapp_prover, two, sok_message)?,
        ))),
    }
    .map(Into::into)
}

fn invalid_bigint_err() -> ExternalSnarkWorkerWorkError {
    ExternalSnarkWorkerWorkError::WorkSpecError(SnarkWorkSpecError::InvalidBigInt)
}

fn prove_single(
    tx_prover: &TransactionProver,
    zkapp_prover: &ZkappProver,
    single: v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single,
    sok_message: &SokMessage,
) -> Result<v2::LedgerProofProdStableV2, ExternalSnarkWorkerWorkError> {
    use ledger::proofs::{merge::MergeParams, transaction::TransactionParams};

    let (snarked_ledger_state, res) = match single {
        v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Transition(
            snarked_ledger_state,
            witness,
        ) => {
            if let v2::MinaTransactionTransactionStableV2::Command(cmd) = &witness.transaction {
                if matches!(&**cmd, v2::MinaBaseUserCommandStableV2::ZkappCommand(_)) {
                    return prove_zkapp(zkapp_prover, snarked_ledger_state, witness, sok_message);
                }
            }
            let res = ledger::proofs::generate_tx_proof(TransactionParams {
                statement: &snarked_ledger_state.0,
                tx_witness: &witness,
                message: sok_message,
                tx_step_prover: &tx_prover.tx_step_prover,
                tx_wrap_prover: &tx_prover.tx_wrap_prover,
                only_verify_constraints: false,
                expected_step_proof: None,
                ocaml_wrap_witness: None,
            });
            (snarked_ledger_state.0, res)
        }
        v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Merge(data) => {
            let (snarked_ledger_state, proof_1, proof_2) = *data;
            let res = ledger::proofs::generate_merge_proof(MergeParams {
                statement: (&snarked_ledger_state.0)
                    .try_into()
                    .map_err(|_| invalid_bigint_err())?,
                proofs: &[proof_1, proof_2],
                message: sok_message,
                step_prover: &tx_prover.merge_step_prover,
                wrap_prover: &tx_prover.tx_wrap_prover,
                only_verify_constraints: false,
                expected_step_proof: None,
                ocaml_wrap_witness: None,
            });
            (snarked_ledger_state.0, res)
        }
    };
    res.map_err(|err| ExternalSnarkWorkerWorkError::Error(err.to_string()))
        .map(|proof| {
            v2::LedgerProofProdStableV2(v2::TransactionSnarkStableV2 {
                statement: v2::MinaStateSnarkedLedgerStateWithSokStableV2 {
                    source: snarked_ledger_state.source,
                    target: snarked_ledger_state.target,
                    connecting_ledger_left: snarked_ledger_state.connecting_ledger_left,
                    connecting_ledger_right: snarked_ledger_state.connecting_ledger_right,
                    supply_increase: snarked_ledger_state.supply_increase,
                    fee_excess: snarked_ledger_state.fee_excess,
                    sok_digest: (&sok_message.digest()).into(),
                },
                proof: v2::TransactionSnarkProofStableV2((&proof).into()),
            })
        })
}

fn prove_zkapp(
    zkapp_prover: &ZkappProver,
    snarked_ledger_state: v2::MinaStateSnarkedLedgerStateStableV2,
    witness: v2::TransactionWitnessStableV2,
    sok_message: &SokMessage,
) -> Result<v2::LedgerProofProdStableV2, ExternalSnarkWorkerWorkError> {
    ledger::proofs::generate_zkapp_proof(ZkappParams {
        statement: &snarked_ledger_state.0,
        tx_witness: &witness,
        message: sok_message,
        step_opt_signed_opt_signed_prover: &zkapp_prover.step_opt_signed_opt_signed_prover,
        step_opt_signed_prover: &zkapp_prover.step_opt_signed_prover,
        step_proof_prover: &zkapp_prover.step_proof_prover,
        merge_step_prover: &zkapp_prover.merge_step_prover,
        tx_wrap_prover: &zkapp_prover.tx_wrap_prover,
        opt_signed_path: None,
        proved_path: None,
    })
    .map(|proof| (&proof).into())
    .map_err(|err| ExternalSnarkWorkerWorkError::Error(err.to_string()))
}

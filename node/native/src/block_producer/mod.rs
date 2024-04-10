mod vrf_evaluator;

use ledger::proofs::{
    block::BlockParams, gates::get_provers, generate_block_proof, transaction::ProofError,
};
use mina_p2p_messages::v2::{
    MinaBaseProofStableV2, ProverExtendBlockchainInputStableV2, StateHash,
};
use node::{
    block_producer::{vrf_evaluator::VrfEvaluatorInput, BlockProducerEvent},
    core::channels::mpsc,
};
use openmina_node_account::AccountSecretKey;

use crate::NodeService;

pub struct BlockProducerService {
    keypair: AccountSecretKey,
    vrf_evaluation_sender: mpsc::UnboundedSender<VrfEvaluatorInput>,
}

impl BlockProducerService {
    pub fn new(
        keypair: AccountSecretKey,
        vrf_evaluation_sender: mpsc::UnboundedSender<VrfEvaluatorInput>,
    ) -> Self {
        Self {
            keypair,
            vrf_evaluation_sender,
        }
    }
}

impl NodeService {
    pub fn block_producer_start(&mut self, producer_keypair: AccountSecretKey) {
        let event_sender = self.event_sender.clone();
        let (vrf_evaluation_sender, vrf_evaluation_receiver) =
            mpsc::unbounded_channel::<VrfEvaluatorInput>();

        self.block_producer = Some(BlockProducerService::new(
            producer_keypair.clone(),
            vrf_evaluation_sender,
        ));

        std::thread::Builder::new()
            .name("openmina_vrf_evaluator".to_owned())
            .spawn(move || {
                vrf_evaluator::vrf_evaluator(
                    event_sender,
                    vrf_evaluation_receiver,
                    producer_keypair.into(),
                );
            })
            .unwrap();
    }
}

pub fn prove(
    input: &ProverExtendBlockchainInputStableV2,
    only_verify_constraints: bool,
) -> Result<Box<MinaBaseProofStableV2>, ProofError> {
    let provers = get_provers();

    let res = generate_block_proof(BlockParams {
        input: &*input,
        block_step_prover: &provers.block_step_prover,
        block_wrap_prover: &provers.block_wrap_prover,
        tx_wrap_prover: &provers.tx_wrap_prover,
        only_verify_constraints,
        expected_step_proof: None,
        ocaml_wrap_witness: None,
    });
    res.map(|proof| MinaBaseProofStableV2((&proof).into()))
        .map(Into::into)
}

impl node::service::BlockProducerService for crate::NodeService {
    fn keypair(&mut self) -> Option<AccountSecretKey> {
        self.block_producer.as_ref().map(|bp| bp.keypair.clone())
    }

    fn prove(&mut self, block_hash: StateHash, input: Box<ProverExtendBlockchainInputStableV2>) {
        if self.replayer.is_some() {
            return;
        }

        let tx = self.event_sender.clone();
        std::thread::spawn(move || {
            let res = prove(&*input, false).map_err(|err| format!("{err:?}"));
            let _ = tx.send(BlockProducerEvent::BlockProve(block_hash, res).into());
        });
    }
}

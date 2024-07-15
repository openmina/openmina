mod vrf_evaluator;

use ledger::proofs::{
    block::BlockParams, gates::get_provers, generate_block_proof, transaction::ProofError,
};
use mina_p2p_messages::v2::{
    MinaBaseProofStableV2, ProverExtendBlockchainInputStableV2, StateHash,
};
use node::{
    account::AccountSecretKey,
    block_producer::{vrf_evaluator::VrfEvaluatorInput, BlockProducerEvent},
    core::{channels::mpsc, constants::constraint_constants},
};

use crate::EventSender;

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

    pub fn start(event_sender: EventSender, keypair: AccountSecretKey) -> Self {
        let (vrf_evaluation_sender, vrf_evaluation_receiver) =
            mpsc::unbounded_channel::<VrfEvaluatorInput>();

        let producer_keypair = keypair.clone();
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

        BlockProducerService::new(keypair, vrf_evaluation_sender)
    }

    pub fn keypair(&self) -> AccountSecretKey {
        self.keypair.clone()
    }
}

pub fn prove(
    mut input: Box<ProverExtendBlockchainInputStableV2>,
    keypair: AccountSecretKey,
    only_verify_constraints: bool,
) -> Result<Box<MinaBaseProofStableV2>, ProofError> {
    let height = input
        .next_state
        .body
        .consensus_state
        .blockchain_length
        .as_u32();
    let is_genesis = height == 1
        || constraint_constants()
            .fork
            .as_ref()
            .map_or(false, |fork| fork.blockchain_length + 1 == height);
    if !is_genesis {
        input.prover_state.producer_private_key = keypair.into();
    }

    let provers = get_provers();

    let res = generate_block_proof(BlockParams {
        input: &input,
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

impl node::service::BlockProducerService for crate::NodeServiceCommon {
    fn prove(&mut self, block_hash: StateHash, input: Box<ProverExtendBlockchainInputStableV2>) {
        if self.replayer.is_some() {
            return;
        }
        let keypair = self.block_producer.as_ref().unwrap().keypair();

        let tx = self.event_sender().clone();
        std::thread::spawn(move || {
            let res = prove(input, keypair, false).map_err(|err| format!("{err:?}"));
            let _ = tx.send(BlockProducerEvent::BlockProve(block_hash, res).into());
        });
    }
}

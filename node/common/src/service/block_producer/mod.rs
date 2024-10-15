mod vrf_evaluator;

use ledger::proofs::{
    block::BlockParams, generate_block_proof, provers::BlockProver, transaction::ProofError,
};
use mina_p2p_messages::{
    binprot::BinProtWrite,
    v2::{MinaBaseProofStableV2, ProverExtendBlockchainInputStableV2, StateHash},
};
use node::{
    account::AccountSecretKey,
    block_producer::{vrf_evaluator::VrfEvaluatorInput, BlockProducerEvent},
    core::{channels::mpsc, constants::constraint_constants, thread},
};

use crate::EventSender;

pub struct BlockProducerService {
    provers: BlockProver,
    keypair: AccountSecretKey,
    vrf_evaluation_sender: mpsc::UnboundedSender<VrfEvaluatorInput>,
}

impl BlockProducerService {
    pub fn new(
        provers: BlockProver,
        keypair: AccountSecretKey,
        vrf_evaluation_sender: mpsc::UnboundedSender<VrfEvaluatorInput>,
    ) -> Self {
        Self {
            provers,
            keypair,
            vrf_evaluation_sender,
        }
    }

    pub fn start(
        provers: BlockProver,
        event_sender: EventSender,
        keypair: AccountSecretKey,
    ) -> Self {
        let (vrf_evaluation_sender, vrf_evaluation_receiver) =
            mpsc::unbounded_channel::<VrfEvaluatorInput>();

        let producer_keypair = keypair.clone();
        thread::Builder::new()
            .name("openmina_vrf_evaluator".to_owned())
            .spawn(move || {
                vrf_evaluator::vrf_evaluator(
                    event_sender,
                    vrf_evaluation_receiver,
                    producer_keypair.into(),
                );
            })
            .unwrap();

        BlockProducerService::new(provers, keypair, vrf_evaluation_sender)
    }

    pub fn keypair(&self) -> AccountSecretKey {
        self.keypair.clone()
    }
}

pub fn prove(
    provers: BlockProver,
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

impl node::service::BlockProducerService for crate::NodeService {
    fn provers(&self) -> BlockProver {
        self.block_producer
            .as_ref()
            .expect("provers shouldn't be needed if block producer isn't initialized")
            .provers
            .clone()
    }

    fn prove(&mut self, block_hash: StateHash, input: Box<ProverExtendBlockchainInputStableV2>) {
        if self.replayer.is_some() {
            return;
        }
        let provers = self.provers();
        let keypair = self.block_producer.as_ref().unwrap().keypair();

        let tx = self.event_sender().clone();
        thread::spawn(move || {
            let res =
                prove(provers, input.clone(), keypair, false).map_err(|err| format!("{err:?}"));
            if res.is_err() {
                // IMPORTANT: Make sure that `input` here is a copy from before `prove` is called, we don't
                // want to leak the private key.
                if let Err(error) = dump_failed_block_proof_input(block_hash.clone(), input) {
                    openmina_core::error!(
                        openmina_core::log::system_time();
                        message = "Failure when dumping failed block proof inputs", error = format!("{error}"));
                }
            }
            let _ = tx.send(BlockProducerEvent::BlockProve(block_hash, res).into());
        });
    }
}

fn dump_failed_block_proof_input(
    block_hash: StateHash,
    input: Box<ProverExtendBlockchainInputStableV2>,
) -> std::io::Result<()> {
    let debug_dir = openmina_core::get_debug_dir();
    let filename = debug_dir
        .join(format!("failed_block_proof_input_{block_hash}.binprot"))
        .to_string_lossy()
        .to_string();
    openmina_core::warn!(
        openmina_core::log::system_time();
        message = "Dumping failed block proof.",
        filename = filename
    );
    std::fs::create_dir_all(&debug_dir)?;
    let mut file = std::fs::File::create(&filename)?;
    input.binprot_write(&mut file)?;
    file.sync_all()?;
    Ok(())
}

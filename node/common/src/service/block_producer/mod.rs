mod vrf_evaluator;

use std::sync::Arc;

use ledger::proofs::{
    block::BlockParams, generate_block_proof, provers::BlockProver, transaction::ProofError,
};
use mina_p2p_messages::{
    bigint::BigInt,
    binprot::{self, BinProtWrite},
    v2::{self, MinaBaseProofStableV2, ProverExtendBlockchainInputStableV2, StateHash},
};
use node::{
    account::AccountSecretKey,
    block_producer::{vrf_evaluator::VrfEvaluatorInput, BlockProducerEvent},
    core::{channels::mpsc, constants::constraint_constants, thread},
};
use rsa::pkcs1::DecodeRsaPublicKey;

use crate::EventSender;

pub struct BlockProducerService {
    provers: Option<BlockProver>,
    keypair: AccountSecretKey,
    vrf_evaluation_sender: mpsc::TrackedUnboundedSender<VrfEvaluatorInput>,
    prove_sender: mpsc::TrackedUnboundedSender<(
        BlockProver,
        StateHash,
        Box<ProverExtendBlockchainInputStableV2>,
    )>,
}

impl BlockProducerService {
    pub fn new(
        keypair: AccountSecretKey,
        vrf_evaluation_sender: mpsc::TrackedUnboundedSender<VrfEvaluatorInput>,
        prove_sender: mpsc::TrackedUnboundedSender<(
            BlockProver,
            StateHash,
            Box<ProverExtendBlockchainInputStableV2>,
        )>,
        provers: Option<BlockProver>,
    ) -> Self {
        Self {
            provers,
            keypair,
            vrf_evaluation_sender,
            prove_sender,
        }
    }

    pub fn start(
        event_sender: EventSender,
        keypair: AccountSecretKey,
        provers: Option<BlockProver>,
    ) -> Self {
        let (vrf_evaluation_sender, vrf_evaluation_receiver) = mpsc::unbounded_channel();
        let (prove_sender, prove_receiver) = mpsc::unbounded_channel();

        let event_sender_clone = event_sender.clone();
        let producer_keypair = keypair.clone();
        thread::Builder::new()
            .name("openmina_vrf_evaluator".to_owned())
            .spawn(move || {
                vrf_evaluator::vrf_evaluator(
                    event_sender_clone,
                    vrf_evaluation_receiver,
                    producer_keypair.into(),
                );
            })
            .unwrap();

        let producer_keypair = keypair.clone();
        thread::Builder::new()
            .name("openmina_block_prover".to_owned())
            .spawn(move || prover_loop(producer_keypair, event_sender, prove_receiver))
            .unwrap();

        BlockProducerService::new(keypair, vrf_evaluation_sender, prove_sender, provers)
    }

    pub fn keypair(&self) -> AccountSecretKey {
        self.keypair.clone()
    }

    pub fn vrf_pending_requests(&self) -> usize {
        self.vrf_evaluation_sender.len()
    }

    pub fn prove_pending_requests(&self) -> usize {
        self.prove_sender.len()
    }
}

fn prover_loop(
    keypair: AccountSecretKey,
    event_sender: EventSender,
    mut rx: mpsc::TrackedUnboundedReceiver<(
        BlockProver,
        StateHash,
        Box<ProverExtendBlockchainInputStableV2>,
    )>,
) {
    while let Some(msg) = rx.blocking_recv() {
        let (provers, block_hash, mut input) = msg.0;
        let res = prove(provers, &mut input, &keypair, false).map_err(|err| format!("{err:?}"));
        if res.is_err() {
            if let Err(error) = dump_failed_block_proof_input(block_hash.clone(), input) {
                openmina_core::error!(
                        openmina_core::log::system_time();
                        message = "Failure when dumping failed block proof inputs", error = format!("{error}"));
            }
        }
        let _ = event_sender.send(BlockProducerEvent::BlockProve(block_hash, res).into());
    }
}

pub fn prove(
    provers: BlockProver,
    input: &mut ProverExtendBlockchainInputStableV2,
    keypair: &AccountSecretKey,
    only_verify_constraints: bool,
) -> Result<Arc<MinaBaseProofStableV2>, ProofError> {
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
            .is_some_and(|fork| fork.blockchain_length + 1 == height);
    if !is_genesis {
        input.prover_state.producer_private_key = keypair.into();
    }

    let res = generate_block_proof(BlockParams {
        input,
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
            .unwrap_or_else(BlockProver::get_once_made)
    }

    fn prove(&mut self, block_hash: StateHash, input: Box<ProverExtendBlockchainInputStableV2>) {
        if self.replayer.is_some() {
            return;
        }
        let provers = self.provers();
        let _ = self
            .block_producer
            .as_ref()
            .expect("prove shouldn't be requested if block producer isn't initialized")
            .prove_sender
            .tracked_send((provers, block_hash, input));
    }

    fn with_producer_keypair<T>(&self, f: impl FnOnce(&AccountSecretKey) -> T) -> Option<T> {
        Some(f(&self.block_producer.as_ref()?.keypair))
    }
}

fn dump_failed_block_proof_input(
    block_hash: StateHash,
    mut input: Box<ProverExtendBlockchainInputStableV2>,
) -> std::io::Result<()> {
    use rsa::Pkcs1v15Encrypt;

    const PUBLIC_KEY: &str = "-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAqVZJX+m/xMB32rMAb9CSh9M4+TGzV037/R7yLCYuLm6VgX0HBtvE
wC7IpZeSQKsc7gx0EVn9+u24nw7ep0TJlJb7bWolRdelnOQK0t9KMn20n8QKYPvA
5zmUXBUI/4Hja+Nck5sErut/PAamzoUK1SeYdbsLRM70GiPALe+buSBb3qEEOgm8
6EYqichDSd1yry2hLy/1EvKm51Va+D92/1SB1TNLFLpUJ6PuSelfYC95wJ+/g+1+
kGqG7QLzSPjAtP/YbUponwaD+t+A0kBg0hV4hhcJOkPeA2NOi04K93bz3HuYCVRe
1fvtAVOmYJ3s4CfRCC3SudYc8ZVvERcylwIDAQAB
-----END RSA PUBLIC KEY-----";

    #[derive(binprot::macros::BinProtWrite)]
    struct DumpBlockProof {
        input: Box<ProverExtendBlockchainInputStableV2>,
        key: Vec<u8>,
    }

    let producer_private_key = {
        let mut buffer = Vec::with_capacity(1024);
        input
            .prover_state
            .producer_private_key
            .binprot_write(&mut buffer)
            .unwrap();
        buffer
    };

    let encrypted_producer_private_key = {
        let mut rng = rand::thread_rng();
        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(PUBLIC_KEY).unwrap();
        public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, &producer_private_key)
            .unwrap()
    };

    // IMPORTANT: Make sure that `input` doesn't leak the private key.
    input.prover_state.producer_private_key = v2::SignatureLibPrivateKeyStableV1(BigInt::one());

    let input = DumpBlockProof {
        input,
        key: encrypted_producer_private_key,
    };

    let debug_dir = openmina_core::get_debug_dir();
    let filename = debug_dir
        .join(format!("failed_block_proof_input_{block_hash}.binprot"))
        .to_string_lossy()
        .to_string();
    openmina_core::warn!(message = "Dumping failed block proof.", filename = filename);
    std::fs::create_dir_all(&debug_dir)?;
    let mut file = std::fs::File::create(&filename)?;
    input.binprot_write(&mut file)?;
    file.sync_all()?;
    Ok(())
}

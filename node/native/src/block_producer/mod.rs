mod vrf_evaluator;

use mina_signer::Keypair;
use node::core::channels::mpsc;

use crate::NodeService;

use vrf::VrfEvaluatorInput;

pub struct BlockProducerService {
    secret_key: Keypair,
    vrf_evaluation_sender: mpsc::UnboundedSender<VrfEvaluatorInput>,
}

impl BlockProducerService {
    pub fn new(secret_key: Keypair, vrf_evaluation_sender: mpsc::UnboundedSender<VrfEvaluatorInput>) -> Self {
        Self {
            secret_key,
            vrf_evaluation_sender,
        }
    }
}

impl NodeService {
    pub fn block_producer_start(&mut self, secret_key: Keypair) {
        let event_sender = self.event_sender.clone();
        let (vrf_evaluation_sender,vrf_evaluation_receiver ) = mpsc::unbounded_channel::<VrfEvaluatorInput>();

        self.block_producer = Some(BlockProducerService::new(secret_key.clone(), vrf_evaluation_sender));

        std::thread::Builder::new()
            .name("openmina_vrf_evaluator".to_owned())
            .spawn(move || {
                vrf_evaluator::vrf_evaluator(event_sender, vrf_evaluation_receiver, secret_key);
            })
            .unwrap();
    }
}

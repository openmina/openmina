mod vrf_evaluator;

use crate::NodeService;

pub struct BlockProducerService {
    secret_key: (),
}

impl NodeService {
    pub fn block_producer_start(&mut self, secret_key: ()) {
        let event_sender = self.event_sender.clone();

        std::thread::Builder::new()
            .name("openmina_vrf_evaluator".to_owned())
            .spawn(move || {
                vrf_evaluator::vrf_evaluator(event_sender);
            })
            .unwrap();
    }
}

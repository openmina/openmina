use node::event_source::Event;
use openmina_core::channels::mpsc::UnboundedSender;

use crate::NodeService;

pub fn vrf_evaluator(event_sender: UnboundedSender<Event>) {
    // TODO(adonagy)
}

impl node::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorService for NodeService {
    fn evaluate(&mut self, data: ()) {
        if let Some(bp) = self.block_producer.as_mut() {
            // TODO(adonagy)
        }
    }
}

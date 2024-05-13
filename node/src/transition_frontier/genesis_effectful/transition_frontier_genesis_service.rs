use std::sync::Arc;

use crate::{service::BlockProducerService, transition_frontier::genesis::GenesisConfig};

pub trait TransitionFrontierGenesisService: BlockProducerService {
    /// Load genesis config and genesis ledger.
    fn load_genesis(&mut self, config: Arc<GenesisConfig>);
}

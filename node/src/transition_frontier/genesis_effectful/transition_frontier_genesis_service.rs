use std::sync::Arc;

use crate::service::BlockProducerService;

use crate::GenesisConfig;

pub trait TransitionFrontierGenesisService: BlockProducerService {
    /// Load genesis config and genesis ledger.
    fn load_genesis(&mut self, config: Arc<GenesisConfig>);
}

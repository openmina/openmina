//! Defines the service interface for genesis block generation,
//! including loading the genesis ledger and proving the genesis block.

use std::sync::Arc;

use crate::{service::BlockProducerService, transition_frontier::genesis::GenesisConfig};

/// Service interface for genesis block generation operations.
pub trait TransitionFrontierGenesisService: BlockProducerService {
    /// Load genesis config and genesis ledger.
    fn load_genesis(&mut self, config: Arc<GenesisConfig>);
}

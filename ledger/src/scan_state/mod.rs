pub mod conv;
pub mod currency;
pub mod fee_excess;
pub mod fee_rate;
mod parallel_scan;
pub mod pending_coinbase;
pub mod protocol_state;
#[allow(clippy::module_inception)]
pub mod scan_state;
pub mod snark_work;
pub mod transaction_logic;
pub mod zkapp_logic;
pub use parallel_scan::SpacePartition;

pub struct GenesisConstant {
    pub protocol: (),
    pub txpool_max_size: usize,
    pub num_accounts: Option<usize>,
    pub zkapp_proof_update_cost: f64,
    pub zkapp_signed_single_update_cost: f64,
    pub zkapp_signed_pair_update_cost: f64,
    pub zkapp_transaction_cost_limit: f64,
    pub max_event_elements: usize,
    pub max_action_elements: usize,
    pub zkapp_cmd_limit_hardcap: usize,
}

// TODO: Not sure if any of those values are correct
pub const GENESIS_CONSTANT: GenesisConstant = GenesisConstant {
    protocol: (),
    txpool_max_size: 3000,
    num_accounts: None,
    zkapp_proof_update_cost: 10.26,
    zkapp_signed_single_update_cost: 9.14,
    zkapp_signed_pair_update_cost: 10.08,
    zkapp_transaction_cost_limit: 69.45,
    max_event_elements: 100,
    max_action_elements: 100,
    zkapp_cmd_limit_hardcap: 128,
};

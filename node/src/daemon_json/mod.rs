use serde::{Deserialize, Serialize};

mod json_genesis;
mod json_ledger;
mod json_daemon;
pub use json_genesis::Genesis;
pub use json_ledger::{
    Account, AccountConfigError, AccountPermissions, AccountTiming, Ledger, Zkapp,
};
pub use json_daemon::Daemon;

/// This type represents a JSON object loaded from daemon.json
/// file. It does not describe its full structure, as it's not
/// necessary for our purpose here. We only need to extract
/// certain information from it.
/// In practice, this format is never really used to convey
/// the network's configuration, as the standard practice is to
/// compile that into the OCaml node's binary.
/// In theory, though, the underlying JSON file could contain
/// more information that the following format describes. If
/// that happens, the format can be extended to accommodate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonJson {
    pub daemon: Option<Daemon>,
    pub ledger: Option<Ledger>,
    pub genesis: Option<Genesis>,
    pub epoch_data: Option<Epochs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epochs {
    pub staking: EpochData,
    pub next: Option<EpochData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochData {
    pub accounts: Option<Vec<Account>>,
    pub hash: Option<String>,
    pub s3_data_hash: Option<String>,
    pub seed: String,
}

#[cfg(test)]
mod test {

    use ledger::{scan_state::currency::Balance, Timing};
    use openmina_node_account::AccountPublicKey;
    use std::str::FromStr;

    use crate::daemon_json::DaemonJson;

    #[test]
    fn test_daemon_json_read() {
        let test_filename = "testing/data/daemon.json";
        println!(
            "Reading from: {}",
            std::env::current_dir().unwrap().display()
        );
        let test_file = std::fs::File::open(test_filename).unwrap();
        let daemon_json: DaemonJson = serde_json::from_reader(test_file).unwrap();
        let ledger = daemon_json.ledger.unwrap();
        assert_eq!(ledger.name, Some("devnet".to_string()));
        let addr1 =
            AccountPublicKey::from_str("B62qnLVz8wM7MfJsuYbjFf4UWbwrUBEL5ZdawExxxFhnGXB6siqokyM")
                .unwrap();
        let acc1 = ledger.find_account(&addr1).unwrap();
        assert_eq!(acc1.balance(), Balance::from_u64(83_333_000_000_000));
        assert_eq!(acc1.delegate().unwrap(), Some(acc1.public_key().unwrap()));
        assert!(acc1.secret_key().unwrap().is_none());
        assert_eq!(acc1.timing().unwrap(), Timing::Untimed);
        let addr2 =
            AccountPublicKey::from_str("B62qnJcRzJpdaXvi6ok3iH7BbP3R6oZtT1C9qTyUr9hNHWRf3eUAJxC")
                .unwrap();
        let acc2 = ledger.find_account(&addr2).unwrap();
        assert_eq!(acc2.balance(), Balance::from_u64(2_000_000_000_000_000));
        assert_eq!(acc2.delegate().unwrap(), Some(acc1.public_key().unwrap()));
        if let Timing::Timed {
            initial_minimum_balance,
            ..
        } = acc2.timing().unwrap()
        {
            assert_eq!(
                initial_minimum_balance,
                Balance::from_u64(1_000_000_000_000_000)
            );
        } else {
            panic!("Expected Timed account");
        }
        let daemon = daemon_json.daemon.unwrap();
        assert_eq!(daemon.tx_pool_max_size(), 3000);
        assert_eq!(daemon.peer_list_url(), None);
        assert_eq!(daemon.slot_tx_end(), None);
        assert_eq!(daemon.slot_chain_end(), None);
    }
}

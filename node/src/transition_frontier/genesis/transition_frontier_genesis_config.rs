use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use crate::{account::AccountSecretKey, daemon_json::EpochData};
use ledger::{
    proofs::caching::{ensure_path_exists, openmina_cache_path},
    scan_state::currency::Balance,
    BaseLedger,
};
use mina_hasher::Fp;
use mina_p2p_messages::{
    binprot::{
        self,
        macros::{BinProtRead, BinProtWrite},
        BinProtRead, BinProtWrite,
    },
    v2::{self, PROTOCOL_CONSTANTS},
};
use openmina_core::constants::constraint_constants;
use serde::{Deserialize, Serialize};

use crate::{
    daemon_json::{self, AccountConfigError, DaemonJson},
    ProtocolConstants,
};

pub use GenesisConfig as TransitionFrontierGenesisConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GenesisConfig {
    Counts {
        whales: usize,
        fish: usize,
        constants: ProtocolConstants,
    },
    BalancesDelegateTable {
        table: Vec<(u64, Vec<u64>)>,
        constants: ProtocolConstants,
    },
    Prebuilt(Cow<'static, [u8]>),
    DaemonJson(Box<DaemonJson>),
    DaemonJsonFile(PathBuf),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenesisConfigLoaded {
    pub constants: ProtocolConstants,
    pub genesis_ledger_hash: v2::LedgerHash,
    pub genesis_total_currency: v2::CurrencyAmountStableV1,
    pub genesis_producer_stake_proof: v2::MinaBaseSparseLedgerBaseStableV2,
    pub staking_epoch_ledger_hash: v2::LedgerHash,
    pub staking_epoch_total_currency: v2::CurrencyAmountStableV1,
    pub staking_epoch_seed: v2::EpochSeed,
    pub next_epoch_ledger_hash: v2::LedgerHash,
    pub next_epoch_total_currency: v2::CurrencyAmountStableV1,
    pub next_epoch_seed: v2::EpochSeed,
}

fn bp_num_delegators(i: usize) -> usize {
    (i + 1) * 2
}

#[derive(Debug, thiserror::Error)]
pub enum GenesisConfigError {
    #[error("no ledger in configuration")]
    NoLedger,
    #[error("declared and computed ledger hashes don't match: {expected} != {computed}")]
    LedgerHashMismatch {
        expected: v2::LedgerHash,
        computed: v2::LedgerHash,
    },
    #[error("account error: {0}")]
    Account(#[from] AccountConfigError),
    #[error("error loading genesis config from precomputed data: {0}")]
    Prebuilt(#[from] binprot::Error),
    #[error("error deserializing daemon.json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl GenesisConfig {
    pub fn load(&self) -> Result<(Vec<ledger::Mask>, GenesisConfigLoaded), GenesisConfigError> {
        Ok(match self {
            Self::Counts {
                whales,
                fish,
                constants,
            } => {
                let (whales, fish) = (*whales, *fish);
                let delegator_balance = |balance: u64| move |i| balance / i as u64;
                let whales = (0..whales).map(|i| {
                    let balance = 8333_u64;
                    let delegators = (1..=bp_num_delegators(i)).map(delegator_balance(50_000_000));
                    (balance, delegators)
                });
                let fish = (0..fish).map(|i| {
                    let balance = 6333_u64;
                    let delegators = (1..=bp_num_delegators(i)).map(delegator_balance(5_000_000));
                    (balance, delegators)
                });
                let delegator_table = whales.chain(fish);
                let (mut mask, genesis_total_currency) =
                    Self::build_ledger_from_balances_delegator_table(delegator_table);
                let genesis_ledger_hash = ledger_hash(&mut mask);
                let staking_epoch_total_currency = genesis_total_currency.clone();
                let next_epoch_total_currency = genesis_total_currency.clone();
                let staking_epoch_seed = v2::EpochSeed::zero();
                let next_epoch_seed = v2::EpochSeed::zero();

                let load_result = GenesisConfigLoaded {
                    constants: constants.clone(),
                    genesis_ledger_hash: genesis_ledger_hash.clone(),
                    genesis_total_currency,
                    genesis_producer_stake_proof: create_genesis_producer_stake_proof(&mask),
                    staking_epoch_ledger_hash: genesis_ledger_hash.clone(),
                    staking_epoch_total_currency,
                    next_epoch_ledger_hash: genesis_ledger_hash,
                    next_epoch_total_currency,
                    staking_epoch_seed,
                    next_epoch_seed,
                };
                let masks = vec![mask];
                (masks, load_result)
            }
            Self::BalancesDelegateTable { table, constants } => {
                let table = table.iter().map(|(bp_balance, delegators)| {
                    let delegators = delegators.iter().copied();
                    (*bp_balance, delegators)
                });
                let (mut mask, genesis_total_currency) =
                    Self::build_ledger_from_balances_delegator_table(table);
                let genesis_ledger_hash = ledger_hash(&mut mask);
                let staking_epoch_total_currency = genesis_total_currency.clone();
                let next_epoch_total_currency = genesis_total_currency.clone();
                let staking_epoch_seed = v2::EpochSeed::zero();
                let next_epoch_seed = v2::EpochSeed::zero();

                let load_result = GenesisConfigLoaded {
                    constants: constants.clone(),
                    genesis_ledger_hash: genesis_ledger_hash.clone(),
                    genesis_total_currency,
                    genesis_producer_stake_proof: create_genesis_producer_stake_proof(&mask),
                    staking_epoch_ledger_hash: genesis_ledger_hash.clone(),
                    staking_epoch_total_currency,
                    next_epoch_ledger_hash: genesis_ledger_hash,
                    next_epoch_total_currency,
                    staking_epoch_seed,
                    next_epoch_seed,
                };
                let masks = vec![mask];
                (masks, load_result)
            }
            Self::Prebuilt(bytes) => {
                let prebuilt = PrebuiltGenesisConfig::read(&mut bytes.as_ref())?;
                prebuilt.load()
            }
            Self::DaemonJson(config) => {
                let mut masks = Vec::new();
                let constants = config
                    .genesis
                    .as_ref()
                    .map_or(PROTOCOL_CONSTANTS, |genesis| genesis.protocol_constants());
                let ledger = config.ledger.as_ref().ok_or(GenesisConfigError::NoLedger)?;
                let accounts = ledger
                    .accounts_with_genesis_winner()
                    .iter()
                    .map(daemon_json::Account::to_account)
                    .collect::<Result<Vec<_>, _>>()?;
                let (mask, total_currency, genesis_ledger_hash) =
                    Self::build_or_load_ledger(ledger.ledger_name(), accounts.into_iter())?;

                masks.push(mask.clone());
                if let Some(expected_hash) = config.ledger.as_ref().and_then(|l| l.hash.as_ref()) {
                    if expected_hash != &genesis_ledger_hash.to_string() {
                        return Err(GenesisConfigError::LedgerHashMismatch {
                            expected: expected_hash.parse().unwrap(),
                            computed: genesis_ledger_hash.clone(),
                        });
                    }
                }

                let staking_epoch_ledger_hash;
                let staking_epoch_total_currency;
                let staking_epoch_seed: v2::EpochSeed;
                let next_epoch_ledger_hash;
                let next_epoch_total_currency;
                let next_epoch_seed: v2::EpochSeed;
                let genesis_producer_stake_proof: v2::MinaBaseSparseLedgerBaseStableV2;
                // TODO(devnet): handle other cases here, right now this works
                // only for the post-fork genesis
                if let Some(data) = &config.epoch_data {
                    let accounts = data
                        .staking
                        .accounts
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(daemon_json::Account::to_account)
                        .collect::<Result<Vec<_>, _>>()?;
                    let (staking_ledger_mask, total_currency, hash) = Self::build_or_load_ledger(
                        data.staking.ledger_name(),
                        accounts.into_iter(),
                    )?;
                    staking_epoch_ledger_hash = hash;
                    staking_epoch_total_currency = total_currency;
                    staking_epoch_seed = v2::EpochSeed::from_str(&data.staking.seed).unwrap();
                    genesis_producer_stake_proof =
                        create_genesis_producer_stake_proof(&staking_ledger_mask);
                    masks.push(staking_ledger_mask);

                    let next = data.next.as_ref().unwrap();
                    let accounts = next
                        .accounts
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(daemon_json::Account::to_account)
                        .collect::<Result<Vec<_>, _>>()?;
                    let (mut _mask, total_currency, hash) =
                        Self::build_or_load_ledger(next.ledger_name(), accounts.into_iter())?;
                    next_epoch_ledger_hash = hash;
                    next_epoch_total_currency = total_currency;
                    next_epoch_seed =
                        v2::EpochSeed::from_str(&data.next.as_ref().unwrap().seed).unwrap();
                } else {
                    staking_epoch_ledger_hash = genesis_ledger_hash.clone();
                    staking_epoch_total_currency = total_currency.clone();
                    staking_epoch_seed = v2::EpochSeed::zero();
                    next_epoch_ledger_hash = genesis_ledger_hash.clone();
                    next_epoch_total_currency = total_currency.clone();
                    next_epoch_seed = v2::EpochSeed::zero();
                    genesis_producer_stake_proof = create_genesis_producer_stake_proof(&mask);
                }

                let result = GenesisConfigLoaded {
                    constants,
                    genesis_ledger_hash,
                    genesis_total_currency: total_currency,
                    genesis_producer_stake_proof,
                    staking_epoch_ledger_hash,
                    staking_epoch_total_currency,
                    next_epoch_ledger_hash,
                    next_epoch_total_currency,
                    staking_epoch_seed,
                    next_epoch_seed,
                };
                (masks, result)
            }
            Self::DaemonJsonFile(path) => {
                let reader = File::open(path)?;
                let c = serde_json::from_reader(reader)?;
                Self::DaemonJson(c).load()?
            }
        })
    }

    fn build_or_load_ledger(
        ledger_name: String,
        accounts: impl Iterator<Item = ledger::Account>,
    ) -> Result<(ledger::Mask, v2::CurrencyAmountStableV1, LedgerHash), GenesisConfigError> {
        openmina_core::info!(
            openmina_core::log::system_time();
            kind = "ledger loading",
            message = "loading the ledger",
            ledger_name = ledger_name,
        );
        match LedgerAccountsWithHash::load(ledger_name)? {
            Some(accounts_with_hash) => {
                let (mask, total_currency) = Self::build_ledger_from_accounts_and_hashes(
                    accounts_with_hash
                        .accounts
                        .iter()
                        .map(ledger::Account::from),
                    accounts_with_hash
                        .hashes
                        .into_iter()
                        .map(|(n, h)| (n, h.to_field()))
                        .collect::<Vec<_>>(),
                );
                openmina_core::info!(
                    openmina_core::log::system_time();
                    kind = "ledger loaded",
                    message = "loaded from cache",
                    ledger_hash = accounts_with_hash.ledger_hash.to_string(),
                );
                Ok((mask, total_currency, accounts_with_hash.ledger_hash))
            }
            None => {
                let (mut mask, total_currency) = Self::build_ledger_from_accounts(accounts);
                let hash = ledger_hash(&mut mask);
                let ledger_accounts = LedgerAccountsWithHash {
                    accounts: mask.fold(Vec::new(), |mut acc, a| {
                        acc.push(a.into());
                        acc
                    }),
                    ledger_hash: hash.clone(),
                    hashes: mask
                        .get_raw_inner_hashes()
                        .into_iter()
                        .map(|(idx, hash)| (idx, v2::LedgerHash::from_fp(hash)))
                        .collect(),
                };
                ledger_accounts.cache()?;

                // let file =
                //     File::create("/tmp/rust-account-hashes.txt").expect("Unable to create file");
                // let mut writer = std::io::BufWriter::new(file);
                //
                // for account in &ledger_accounts.accounts {
                //     let account = ledger::Account::from(account);
                //     let hash = v2::LedgerHash::from_fp(account.hash());
                //     let pubkey = &account.public_key.into_address();
                //     let line = format!("{} {}\n", pubkey, hash.to_string());
                //     writer
                //         .write_all(line.as_bytes())
                //         .expect("Unable to write data");
                // }
                // writer.flush().expect("Unable to flush data");
                openmina_core::info!(
                    openmina_core::log::system_time();
                    kind = "ledger loaded",
                    message = "built from config and cached",
                    ledger_hash = hash.to_string(),
                );
                Ok((mask, total_currency, hash))
            }
        }
    }

    fn build_ledger_from_balances_delegator_table(
        block_producers: impl IntoIterator<Item = (u64, impl IntoIterator<Item = u64>)>,
    ) -> (ledger::Mask, v2::CurrencyAmountStableV1) {
        let i = std::rc::Rc::new(std::cell::RefCell::new(0));
        let accounts = block_producers
            .into_iter()
            .flat_map(move |(bp_balance, delegators)| {
                *i.borrow_mut() += 1;
                let bp_sec_key = AccountSecretKey::deterministic(*i.borrow());
                let bp_pub_key: mina_signer::CompressedPubKey = bp_sec_key.public_key().into();

                let account_id = ledger::AccountId::new(bp_pub_key.clone(), Default::default());
                let account = ledger::Account::create_with(
                    account_id,
                    Balance::from_mina(bp_balance).unwrap(),
                );

                let i = i.clone();
                let delegators = delegators.into_iter().map(move |balance| {
                    *i.borrow_mut() += 1;
                    let sec_key = AccountSecretKey::deterministic(*i.borrow());
                    let pub_key = sec_key.public_key();
                    let account_id = ledger::AccountId::new(pub_key.into(), Default::default());
                    let mut account = ledger::Account::create_with(
                        account_id,
                        Balance::from_mina(balance).unwrap(),
                    );
                    account.delegate = Some(bp_pub_key.clone());
                    account
                });

                std::iter::once(account).chain(delegators)
            });
        let accounts = genesis_account_iter().chain(accounts);
        Self::build_ledger_from_accounts(accounts)
    }

    fn build_ledger_from_accounts(
        accounts: impl IntoIterator<Item = ledger::Account>,
    ) -> (ledger::Mask, v2::CurrencyAmountStableV1) {
        let db = ledger::Database::create(constraint_constants().ledger_depth as u8);
        let mask = ledger::Mask::new_root(db);
        let (mask, total_currency) =
            accounts
                .into_iter()
                .fold((mask, 0), |(mut mask, mut total_currency), account| {
                    let account_id = account.id();
                    total_currency += account.balance.as_u64();
                    mask.get_or_create_account(account_id, account).unwrap();
                    (mask, total_currency)
                });

        (mask, v2::CurrencyAmountStableV1(total_currency.into()))
    }

    fn build_ledger_from_accounts_and_hashes(
        accounts: impl IntoIterator<Item = ledger::Account>,
        hashes: Vec<(u64, Fp)>,
    ) -> (ledger::Mask, v2::CurrencyAmountStableV1) {
        let (mask, total_currency) = Self::build_ledger_from_accounts(accounts);

        // Must happen after the accounts have been set to avoid
        // cache invalidations.
        mask.set_raw_inner_hashes(hashes);

        (mask, total_currency)
    }
}

fn ledger_hash(mask: &mut ledger::Mask) -> v2::LedgerHash {
    v2::MinaBaseLedgerHash0StableV1(mask.merkle_root().into()).into()
}

fn genesis_account_iter() -> impl Iterator<Item = ledger::Account> {
    std::iter::once({
        // add genesis producer as the first account.
        let pub_key = AccountSecretKey::genesis_producer().public_key();
        let account_id = ledger::AccountId::new(pub_key.into(), Default::default());
        ledger::Account::create_with(account_id, Balance::from_u64(0))
    })
}

fn create_genesis_producer_stake_proof(
    mask: &ledger::Mask,
) -> v2::MinaBaseSparseLedgerBaseStableV2 {
    let producer = AccountSecretKey::genesis_producer().public_key();
    let producer_id = ledger::AccountId::new(producer.into(), ledger::TokenId::default());
    let sparse_ledger =
        ledger::sparse_ledger::SparseLedger::of_ledger_subset_exn(mask.clone(), &[producer_id]);
    (&sparse_ledger).into()
}

#[derive(Debug, BinProtRead, BinProtWrite)]
struct LedgerAccountsWithHash {
    ledger_hash: LedgerHash,
    accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    hashes: Vec<(u64, LedgerHash)>,
}

impl LedgerAccountsWithHash {
    fn cache(&self) -> Result<(), std::io::Error> {
        let cache_dir = openmina_cache_path("ledgers").unwrap();
        let cache_file = cache_dir.join(format!("{}.bin", self.ledger_hash));
        ensure_path_exists(cache_dir)?;
        let mut file = File::create(cache_file)?;
        self.binprot_write(&mut file)
    }

    fn load(ledger_name: String) -> Result<Option<Self>, binprot::Error> {
        let cache_filename = openmina_cache_path(format!("ledgers/{}.bin", ledger_name)).unwrap();
        if cache_filename.is_file() {
            let mut file = File::open(cache_filename)?;
            LedgerAccountsWithHash::binprot_read(&mut file).map(Some)
        } else {
            Ok(None)
        }
    }
}

use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};

/// Precalculated genesis configuration.
#[derive(Debug, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PrebuiltGenesisConfig {
    constants: ProtocolConstants,
    accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ledger_hash: LedgerHash,
    hashes: Vec<(u64, LedgerHash)>,
    staking_epoch_data: PrebuiltGenesisEpochData,
    next_epoch_data: PrebuiltGenesisEpochData,
}

impl PrebuiltGenesisConfig {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, binprot::Error> {
        PrebuiltGenesisConfig::binprot_read(&mut reader)
    }

    pub fn store<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        self.binprot_write(&mut writer)
    }

    pub fn load(self) -> (Vec<ledger::Mask>, GenesisConfigLoaded) {
        let mut masks = Vec::new();
        let (mask, genesis_total_currency) = GenesisConfig::build_ledger_from_accounts_and_hashes(
            self.accounts.into_iter().map(|acc| (&acc).into()),
            self.hashes
                .into_iter()
                .map(|(n, h)| (n, h.to_field()))
                .collect::<Vec<_>>(),
        );
        masks.push(mask);
        let (staking_ledger_mask, staking_epoch_total_currency) =
            GenesisConfig::build_ledger_from_accounts_and_hashes(
                self.staking_epoch_data
                    .accounts
                    .into_iter()
                    .map(|acc| (&acc).into()),
                self.staking_epoch_data
                    .hashes
                    .into_iter()
                    .map(|(n, h)| (n, h.to_field()))
                    .collect::<Vec<_>>(),
            );
        let (_mask, next_epoch_total_currency) =
            GenesisConfig::build_ledger_from_accounts_and_hashes(
                self.next_epoch_data
                    .accounts
                    .into_iter()
                    .map(|acc| (&acc).into()),
                self.next_epoch_data
                    .hashes
                    .into_iter()
                    .map(|(n, h)| (n, h.to_field()))
                    .collect::<Vec<_>>(),
            );

        let load_result = GenesisConfigLoaded {
            constants: self.constants,
            genesis_ledger_hash: self.ledger_hash,
            genesis_total_currency,
            genesis_producer_stake_proof: create_genesis_producer_stake_proof(&staking_ledger_mask),
            staking_epoch_ledger_hash: self.staking_epoch_data.ledger_hash.clone(),
            staking_epoch_total_currency,
            next_epoch_ledger_hash: self.next_epoch_data.ledger_hash.clone(),
            next_epoch_total_currency,
            staking_epoch_seed: self.staking_epoch_data.seed,
            next_epoch_seed: self.next_epoch_data.seed,
        };
        masks.push(staking_ledger_mask);
        (masks, load_result)
    }
}

#[derive(Debug, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct PrebuiltGenesisEpochData {
    accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ledger_hash: LedgerHash,
    hashes: Vec<(u64, LedgerHash)>,
    seed: v2::EpochSeed,
}

impl TryFrom<DaemonJson> for PrebuiltGenesisConfig {
    type Error = GenesisConfigError;

    fn try_from(config: DaemonJson) -> Result<Self, Self::Error> {
        let constants = config
            .genesis
            .as_ref()
            .map_or(PROTOCOL_CONSTANTS, |genesis| genesis.protocol_constants());
        let ledger = config.ledger.as_ref().ok_or(GenesisConfigError::NoLedger)?;
        let ledger_accounts = ledger
            .accounts_with_genesis_winner()
            .iter()
            .map(daemon_json::Account::to_account)
            .collect::<Result<Vec<_>, _>>()?;
        let accounts = ledger_accounts.iter().map(Into::into).collect();
        let (mut mask, _total_currency) =
            GenesisConfig::build_ledger_from_accounts(ledger_accounts);
        let ledger_hash = ledger_hash(&mut mask);
        let hashes = mask
            .get_raw_inner_hashes()
            .into_iter()
            .map(|(idx, hash)| (idx, v2::LedgerHash::from_fp(hash)))
            .collect();
        let daemon_json::Epochs { staking, next } = config.epoch_data.unwrap();
        let staking_epoch_data = staking.try_into().unwrap();
        let next_epoch_data = next.unwrap().try_into().unwrap();
        let result = PrebuiltGenesisConfig {
            constants,
            accounts,
            ledger_hash,
            hashes,
            staking_epoch_data,
            next_epoch_data,
        };
        Ok(result)
    }
}

impl TryFrom<EpochData> for PrebuiltGenesisEpochData {
    type Error = GenesisConfigError;

    fn try_from(value: EpochData) -> Result<Self, Self::Error> {
        let EpochData {
            accounts,
            hash,
            s3_data_hash: _,
            seed,
        } = value;

        let expected_ledger_hash = hash.unwrap().parse().unwrap();
        let seed = seed.parse().unwrap();
        let ledger_accounts = accounts
            .unwrap()
            .iter()
            .map(daemon_json::Account::to_account)
            .collect::<Result<Vec<_>, _>>()?;
        let accounts = ledger_accounts.iter().map(Into::into).collect();
        let (mut mask, _total_currency) =
            GenesisConfig::build_ledger_from_accounts(ledger_accounts);
        let ledger_hash = ledger_hash(&mut mask);

        if ledger_hash != expected_ledger_hash {
            return Err(Self::Error::LedgerHashMismatch {
                expected: expected_ledger_hash,
                computed: ledger_hash,
            });
        }

        let hashes = mask
            .get_raw_inner_hashes()
            .into_iter()
            .map(|(idx, hash)| (idx, v2::LedgerHash::from_fp(hash)))
            .collect();

        Ok(Self {
            accounts,
            ledger_hash,
            hashes,
            seed,
        })
    }
}

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
use openmina_core::constants::{constraint_constants, DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS};
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
        non_stakers: NonStakers,
        constants: ProtocolConstants,
    },
    BalancesDelegateTable {
        table: Vec<(u64, Vec<u64>)>,
        constants: ProtocolConstants,
    },
    AccountsBinProt {
        bytes: Cow<'static, [u8]>,
        constants: ProtocolConstants,
    },
    Prebuilt(Cow<'static, [u8]>),
    DaemonJson(Box<DaemonJson>),
    DaemonJsonFile(PathBuf),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NonStakers {
    /// Adds all the accounts from the generated array to the ledger
    Fill,
    /// No non-staker accounts will be added to the ledger
    None,
    /// Add a precise amount of accounts, non greater than the amount of generated accounts
    Count(usize),
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
    pub fn default_constants(timestamp_ms: u64) -> ProtocolConstants {
        ProtocolConstants {
            k: 290.into(),
            slots_per_epoch: 7140.into(),
            slots_per_sub_window: 7.into(),
            grace_period_slots: 2160.into(),
            delta: 0.into(),
            genesis_state_timestamp: v2::BlockTimeTimeStableV1(
                v2::UnsignedExtendedUInt64Int64ForVersionTagsStableV1(timestamp_ms.into()),
            ),
        }
    }

    // This is a stub for the moment until PR #420 is merged, which implements this for
    // real. In case of conflict, delete this stub and put the real implementation here.
    pub fn protocol_constants(&self) -> Result<ProtocolConstants, time::error::Parse> {
        match self {
            Self::Counts { constants, .. }
            | Self::BalancesDelegateTable { constants, .. }
            | Self::AccountsBinProt { constants, .. } => Ok(constants.clone()),
            Self::Prebuilt { .. } => Ok(self.load().unwrap().1.constants),
            Self::DaemonJson(config) => Ok(config
                .genesis
                .as_ref()
                .map(|g: &daemon_json::Genesis| g.protocol_constants())
                .unwrap_or(Self::default_constants(
                    DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS,
                ))),
            Self::DaemonJsonFile(_) => todo!(),
        }
    }

    pub fn load(
        &self,
    ) -> anyhow::Result<(Vec<ledger::Mask>, GenesisConfigLoaded), GenesisConfigError> {
        Ok(match self {
            Self::Counts {
                whales,
                fish,
                non_stakers,
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
                    Self::build_ledger_from_balances_delegator_table(delegator_table, non_stakers);
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
                    Self::build_ledger_from_balances_delegator_table(table, &NonStakers::None);
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
            Self::AccountsBinProt { bytes, constants } => {
                let mut bytes = bytes.as_ref();
                let expected_hash = Option::<v2::LedgerHash>::binprot_read(&mut bytes)?;
                let hashes = Vec::<(u64, v2::LedgerHash)>::binprot_read(&mut bytes)?
                    .into_iter()
                    .map(|(idx, hash)| (idx, hash.0.to_field()))
                    .collect();
                let accounts = Vec::<ledger::Account>::binprot_read(&mut bytes)?;

                let (mut mask, total_currency) =
                    Self::build_ledger_from_accounts_and_hashes(accounts, hashes);
                let ledger_hash = ledger_hash(&mut mask);

                todo!()

                // // TODO(tizoc): currently this doesn't really do much, because now we load the hashes
                // // from the bin_prot data too to speed up the loading. Maybe add some flag
                // // to force the rehashing and validation of the loaded ledger hashes.
                // if let Some(expected_hash) = expected_hash.filter(|h| h != &ledger_hash) {
                //     anyhow::bail!("ledger hash mismatch after building the mask! expected: '{expected_hash}', got '{ledger_hash}'");
                // }
                //
                // let load_result = GenesisConfigLoaded {
                //     constants: constants.clone(),
                //     ledger_hash,
                //     total_currency,
                //     genesis_producer_stake_proof: genesis_producer_stake_proof(&mask),
                // };
                // (mask, load_result)
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
        non_stakers: &NonStakers,
    ) -> (ledger::Mask, v2::CurrencyAmountStableV1) {
        let mut counter = 0;
        let mut total_balance = 0;

        fn create_account(
            counter: &mut u64,
            balance: u64,
            delegate: Option<mina_signer::CompressedPubKey>,
            total_balance: &mut u64,
        ) -> ledger::Account {
            let sec_key = AccountSecretKey::deterministic(*counter);
            let pub_key: mina_signer::CompressedPubKey = sec_key.public_key().into();
            let account_id = ledger::AccountId::new(pub_key.clone(), Default::default());
            let mut account =
                ledger::Account::create_with(account_id, Balance::from_mina(balance).unwrap());
            account.delegate = delegate;
            *total_balance += balance;
            *counter += 1;
            // println!("Created account with balance: {}, total_balance: {}", balance, *total_balance); // Debug print
            account
        }

        let mut accounts = Vec::new();

        // Process block producers and their delegators
        for (bp_balance, delegators) in block_producers {
            let bp_account = create_account(&mut counter, bp_balance, None, &mut total_balance);
            let bp_pub_key = bp_account.public_key.clone();
            accounts.push(bp_account);

            for balance in delegators {
                let delegator_account = create_account(
                    &mut counter,
                    balance,
                    Some(bp_pub_key.clone()),
                    &mut total_balance,
                );
                accounts.push(delegator_account);
            }
        }

        let remaining_accounts = AccountSecretKey::max_deterministic_count()
            .checked_sub(counter as usize)
            .unwrap_or_default();

        let non_staker_count = match non_stakers {
            NonStakers::Fill => remaining_accounts,
            NonStakers::None => 0,
            NonStakers::Count(count) => *std::cmp::min(count, &remaining_accounts),
        };

        let non_staker_total = total_balance * 20 / 80;
        let non_staker_balance = if non_staker_count > 0 {
            non_staker_total / non_staker_count as u64
        } else {
            0
        };

        println!("Non staker total balance: {}", non_staker_total);

        // Process non-stakers
        if matches!(non_stakers, NonStakers::Fill | NonStakers::Count(_)) {
            for _ in 0..non_staker_count {
                let non_staker_account =
                    create_account(&mut counter, non_staker_balance, None, &mut total_balance);
                accounts.push(non_staker_account);
            }
        }

        // Add genesis accounts
        for genesis_account in genesis_account_iter() {
            accounts.push(genesis_account);
        }

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

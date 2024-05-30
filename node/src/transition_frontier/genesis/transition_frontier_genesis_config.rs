use std::borrow::Cow;

use crate::account::AccountSecretKey;
use ledger::{scan_state::currency::Balance, Account, BaseLedger};
use mina_hasher::Fp;
use mina_p2p_messages::{binprot::BinProtRead, v2};
use openmina_core::constants::{CONSTRAINT_CONSTANTS, DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS};
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
    AccountsBinProt {
        bytes: Cow<'static, [u8]>,
        constants: ProtocolConstants,
    },
    DaemonJson(DaemonJson),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenesisConfigLoaded {
    pub constants: ProtocolConstants,
    pub ledger_hash: v2::LedgerHash,
    pub total_currency: v2::CurrencyAmountStableV1,
    pub genesis_producer_stake_proof: v2::MinaBaseSparseLedgerBaseStableV2,
}

fn bp_num_delegators(i: usize) -> usize {
    (i + 1) * 2
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

    pub fn protocol_constants(&self) -> Result<ProtocolConstants, time::error::Parse> {
        match self {
            Self::Counts { constants, .. }
            | Self::BalancesDelegateTable { constants, .. }
            | Self::AccountsBinProt { constants, .. } => Ok(constants.clone()),
            Self::DaemonJson(config) => {
                let genesis_timestamp = config
                    .genesis
                    .as_ref()
                    .map(|g: &daemon_json::Genesis| g.genesis_state_timestamp().map(|t| t.0 .0 .0))
                    .transpose()?
                    .unwrap_or(DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS);
                Ok(Self::default_constants(genesis_timestamp))
            }
        }
    }

    pub fn load(&self) -> anyhow::Result<(ledger::Mask, GenesisConfigLoaded)> {
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
                let (mut mask, total_currency) =
                    Self::build_ledger_from_balances_delegator_table(delegator_table);

                let load_result = GenesisConfigLoaded {
                    constants: constants.clone(),
                    ledger_hash: ledger_hash(&mut mask),
                    total_currency,
                    genesis_producer_stake_proof: genesis_producer_stake_proof(&mask),
                };
                (mask, load_result)
            }
            Self::BalancesDelegateTable { table, constants } => {
                let table = table.iter().map(|(bp_balance, delegators)| {
                    let delegators = delegators.iter().copied();
                    (*bp_balance, delegators)
                });
                let (mut mask, total_currency) =
                    Self::build_ledger_from_balances_delegator_table(table);

                let load_result = GenesisConfigLoaded {
                    constants: constants.clone(),
                    ledger_hash: ledger_hash(&mut mask),
                    total_currency,
                    genesis_producer_stake_proof: genesis_producer_stake_proof(&mask),
                };
                (mask, load_result)
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

                // TODO(tizoc): currently this doesn't really do much, because now we load the hashes
                // from the bin_prot data too to speed up the loading. Maybe add some flag
                // to force the rehashing and validation of the loaded ledger hashes.
                if let Some(expected_hash) = expected_hash.filter(|h| h != &ledger_hash) {
                    anyhow::bail!("ledger hash mismatch after building the mask! expected: '{expected_hash}', got '{ledger_hash}'");
                }

                let load_result = GenesisConfigLoaded {
                    constants: constants.clone(),
                    ledger_hash,
                    total_currency,
                    genesis_producer_stake_proof: genesis_producer_stake_proof(&mask),
                };
                (mask, load_result)
            }
            Self::DaemonJson(config) => {
                let constants = self.protocol_constants()?;
                let ledger = config
                    .ledger
                    .as_ref()
                    .ok_or(anyhow::anyhow!("No ledger in config"))?;
                let accounts: Vec<Account> = ledger
                    .accounts_with_genesis_winner()
                    .iter()
                    .map(daemon_json::Account::to_account)
                    .collect::<Result<Vec<Account>, AccountConfigError>>()
                    .map_err(anyhow::Error::from)?;
                let (mut mask, total_currency) = Self::build_ledger_from_accounts(accounts);
                let result = GenesisConfigLoaded {
                    constants,
                    ledger_hash: ledger_hash(&mut mask),
                    total_currency,
                    genesis_producer_stake_proof: genesis_producer_stake_proof(&mask),
                };
                (mask, result)
            }
        })
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
        let db = ledger::Database::create(CONSTRAINT_CONSTANTS.ledger_depth as u8);
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

fn genesis_producer_stake_proof(mask: &ledger::Mask) -> v2::MinaBaseSparseLedgerBaseStableV2 {
    let producer = AccountSecretKey::genesis_producer().public_key();
    let producer_id = ledger::AccountId::new(producer.into(), ledger::TokenId::default());
    let sparse_ledger =
        ledger::sparse_ledger::SparseLedger::of_ledger_subset_exn(mask.clone(), &[producer_id]);
    (&sparse_ledger).into()
}

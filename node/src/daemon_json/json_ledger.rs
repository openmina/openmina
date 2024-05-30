use core::str::FromStr;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt::{self, Display, Formatter};

use ledger::{
    scan_state::currency::{Amount, Balance, Magnitude, Nonce, Slot, SlotSpan, TxnVersion},
    AuthRequired, Permissions, ReceiptChainHash, SetVerificationKey, Timing, TokenId, TokenSymbol,
    VotingFor, ZkAppAccount, ZkAppUri,
};
use openmina_node_account::{AccountPublicKey, AccountSecretKey};

type RawCurrency = String;
type RawSlot = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub accounts: Option<Vec<Account>>,
    pub num_accounts: Option<usize>,
    pub balances: Option<Vec<(usize, RawCurrency)>>,
    pub hash: Option<String>,
    pub s3_data_hash: Option<String>,
    pub name: Option<String>,
    pub add_genesis_winner: Option<bool>,
}

impl Ledger {
    pub fn find_account(&self, pk: &AccountPublicKey) -> Option<Account> {
        let pk_str = pk.to_string();
        self.accounts.as_ref().and_then(|accounts| {
            accounts
                .iter()
                .find(|account| account.pk == pk_str)
                .cloned()
        })
    }

    // Add the genesis winner account if the config says it should be added and also
    // if its not already present at the head of the accounts list.
    pub fn accounts_with_genesis_winner(&self) -> Vec<Account> {
        let mut accounts = match self.accounts {
            Some(ref accounts) => accounts.clone(),
            None => Vec::with_capacity(1),
        };
        if !self.add_genesis_winner.unwrap_or(true) {
            return accounts;
        }
        let genesis_winner_pk =
            "B62qiy32p8kAKnny8ZFwoMhYpBppM1DWVCqAPBYNcXnsAHhnfAAuXgg".to_string();
        match accounts.first() {
            Some(first) if first.pk == genesis_winner_pk => accounts,
            _ => {
                let genesis_winner = Account::new(
                    genesis_winner_pk.clone(),
                    "0.000001000".to_string(),
                    Some(genesis_winner_pk),
                );
                accounts.insert(0, genesis_winner);
                accounts
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pk: String,
    sk: Option<String>,
    balance: RawCurrency,
    delegate: Option<String>,
    token_id: Option<String>,
    token_symbol: Option<String>,
    nonce: Option<u32>,
    receipt_chain_hash: Option<String>,
    voting_for: Option<String>,
    timing: Option<AccountTiming>,
    permissions: Option<AccountPermissions>,
    zkapp: Option<Zkapp>,
}

impl Account {
    pub fn new(pk: String, balance: RawCurrency, delegate: Option<String>) -> Account {
        Account {
            pk,
            sk: None,
            balance,
            delegate,
            token_id: None,
            token_symbol: None,
            nonce: None,
            receipt_chain_hash: None,
            voting_for: None,
            timing: None,
            permissions: None,
            zkapp: None,
        }
    }
}

impl Account {
    pub fn public_key(&self) -> Result<AccountPublicKey, AccountConfigError> {
        let cpk = CompressedPubKey::from_address(&self.pk)
            .map_err(|_| AccountConfigError::MalformedKey(self.pk.clone()))?;
        Ok(AccountPublicKey::from(cpk))
    }

    pub fn balance(&self) -> Balance {
        Balance::of_mina_string_exn(&self.balance)
    }

    pub fn delegate(&self) -> Result<Option<AccountPublicKey>, AccountConfigError> {
        let is_default_token = self.token_id()?.is_default();
        match self.delegate.as_ref() {
            Some(delegate) if is_default_token => {
                let cpk = CompressedPubKey::from_address(delegate)
                    .map_err(|_| AccountConfigError::MalformedKey(delegate.clone()))?;
                Ok(Some(AccountPublicKey::from(cpk)))
            }
            Some(_) => Err(AccountConfigError::DelegateSetOnNonDefaultTokenAccount),
            None => {
                if is_default_token {
                    self.public_key().map(Option::Some)
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn secret_key(&self) -> Result<Option<AccountSecretKey>, AccountConfigError> {
        match self.sk.as_ref() {
            Some(sk) => AccountSecretKey::from_str(sk)
                .map(Some)
                .map_err(|_| AccountConfigError::MalformedKey(sk.to_string())),
            None => Ok(None),
        }
    }

    pub fn token_id(&self) -> Result<TokenId, AccountConfigError> {
        let token_fp = self
            .token_id
            .as_ref()
            .map(|id| {
                Fp::from_str(id).map_err(|()| AccountConfigError::MalformedTokenId(id.clone()))
            })
            .transpose()?;
        Ok(token_fp.map_or(TokenId::default(), TokenId))
    }

    pub fn token_symbol(&self) -> TokenSymbol {
        self.token_symbol
            .clone()
            .map(TokenSymbol)
            .unwrap_or_default()
    }

    pub fn nonce(&self) -> Nonce {
        self.nonce.map_or(Nonce::zero(), Nonce::from_u32)
    }

    pub fn receipt_chain_hash(&self) -> Result<ReceiptChainHash, AccountConfigError> {
        self.receipt_chain_hash
            .as_ref()
            .map_or(Ok(ReceiptChainHash::empty()), |hash| {
                Fp::from_str(hash)
                    .map(ReceiptChainHash)
                    .map_err(|_| AccountConfigError::MalformedReceiptChainHash(hash.clone()))
            })
    }

    pub fn voting_for(&self) -> Result<VotingFor, AccountConfigError> {
        self.voting_for
            .as_ref()
            .map_or(Ok(VotingFor::dummy()), |hash| {
                Fp::from_str(hash)
                    .map(VotingFor)
                    .map_err(|_| AccountConfigError::MalformedVotingFor(hash.clone()))
            })
    }

    pub fn timing(&self) -> Result<Timing, AccountConfigError> {
        self.timing
            .as_ref()
            .map_or(Ok(Timing::Untimed), AccountTiming::to_timing)
    }

    pub fn permissions(&self) -> Permissions<AuthRequired> {
        self.permissions.as_ref().map_or(
            Permissions::user_default(),
            AccountPermissions::to_permissions,
        )
    }

    pub fn zkapp(&self) -> Result<Option<ZkAppAccount>, AccountConfigError> {
        self.zkapp.as_ref().map(Zkapp::to_zkapp_account).transpose()
    }

    pub fn to_account(&self) -> Result<ledger::Account, AccountConfigError> {
        let mut account = ledger::Account::empty();
        account.public_key = self.public_key()?.into();
        account.token_id = self.token_id()?;
        account.token_symbol = self.token_symbol();
        account.balance = self.balance();
        account.nonce = self.nonce();
        account.receipt_chain_hash = self.receipt_chain_hash()?;
        account.delegate = self.delegate()?.map(|pk| pk.into());
        account.voting_for = self.voting_for()?;
        account.timing = self.timing()?;
        account.permissions = self.permissions();
        account.zkapp = self.zkapp()?;
        Ok(account)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTiming {
    initial_minimum_balance: RawCurrency,
    cliff_time: GlobalSlotSinceGenesis,
    cliff_amount: RawCurrency,
    vesting_period: GlobalSlotSpan,
    vesting_increment: RawCurrency,
}

impl AccountTiming {
    fn to_timing(&self) -> Result<Timing, AccountConfigError> {
        let initial_minimum_balance = Balance::of_mina_string_exn(&self.initial_minimum_balance);
        let GlobalSlotSinceGenesis(cliff_time) = self.cliff_time;
        let cliff_amount = Amount::of_mina_string_exn(&self.cliff_amount);
        let GlobalSlotSpan(vesting_period) = self.vesting_period;
        let vesting_increment = Amount::of_mina_string_exn(&self.vesting_increment);
        Ok(Timing::Timed {
            initial_minimum_balance,
            cliff_time: Slot::from_u32(cliff_time),
            cliff_amount,
            vesting_period: SlotSpan::from_u32(vesting_period),
            vesting_increment,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerVrfKeyPerm {
    auth: AuthRequired,
    txn_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPermissions {
    access: AuthRequired,
    edit_state: AuthRequired,
    send: AuthRequired,
    receive: AuthRequired,
    set_delegate: AuthRequired,
    set_permissions: AuthRequired,
    set_verification_key: SerVrfKeyPerm,
    set_zkapp_uri: AuthRequired,
    edit_action_state: AuthRequired,
    set_token_symbol: AuthRequired,
    increment_nonce: AuthRequired,
    set_voting_for: AuthRequired,
    set_timing: AuthRequired,
}

impl AccountPermissions {
    fn to_permissions(&self) -> Permissions<AuthRequired> {
        Permissions {
            access: self.access,
            edit_state: self.edit_state,
            send: self.send,
            receive: self.receive,
            set_delegate: self.set_delegate,
            set_permissions: self.set_permissions,
            set_verification_key: SetVerificationKey {
                auth: self.set_verification_key.auth,
                txn_version: TxnVersion::from_u32(self.set_verification_key.txn_version),
            },
            set_zkapp_uri: self.set_zkapp_uri,
            edit_action_state: self.edit_action_state,
            set_token_symbol: self.set_token_symbol,
            increment_nonce: self.increment_nonce,
            set_voting_for: self.set_voting_for,
            set_timing: self.set_timing,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zkapp {
    app_state: Vec<String>,
    // These verification keys currently don't appear in configs, but
    // they could in the future. We ignore them for now, but should they
    // appear, serialization is likely to crash and we'll have to update
    // this type and relevant conversion functions.
    verification_key: Option<()>,
    zkapp_version: u32,
    action_state: Vec<String>,
    last_action_slot: RawSlot,
    proved_state: bool,
    zkapp_uri: String,
}

fn parse_fp(str: &str) -> Result<Fp, AccountConfigError> {
    Fp::from_str(str).map_err(|_| AccountConfigError::MalformedFp(str.to_owned()))
}

impl Zkapp {
    fn to_zkapp_account(&self) -> Result<ZkAppAccount, AccountConfigError> {
        let app_state_fps: Vec<Fp> = self
            .app_state
            .iter()
            .map(|fp| parse_fp(fp))
            .collect::<Result<Vec<Fp>, AccountConfigError>>()?;
        let app_state = if app_state_fps.len() <= 8 {
            let mut app_state = [0.into(); 8];
            app_state.copy_from_slice(&app_state_fps);
            app_state
        } else {
            return Err(AccountConfigError::ZkAppStateTooLong(
                self.app_state.clone(),
            ));
        };
        let act_state_fps: Vec<Fp> = self
            .action_state
            .iter()
            .map(|fp| parse_fp(fp))
            .collect::<Result<Vec<Fp>, AccountConfigError>>()?;
        let action_state = if act_state_fps.len() <= 5 {
            let mut action_state = [0.into(); 5];
            action_state.copy_from_slice(&act_state_fps);
            action_state
        } else {
            return Err(AccountConfigError::ZkAppStateTooLong(
                self.action_state.clone(),
            ));
        };
        let last_action_slot = self
            .last_action_slot
            .parse::<u32>()
            .map(Slot::from_u32)
            .map_err(|_| AccountConfigError::MalformedSlot(self.last_action_slot.clone()))?;
        if self.verification_key.is_some() {
            return Err(AccountConfigError::VerificationKeyParsingNotSupported);
        };
        Ok(ZkAppAccount {
            app_state,
            verification_key: None,
            zkapp_version: self.zkapp_version,
            action_state,
            last_action_slot,
            proved_state: self.proved_state,
            zkapp_uri: ZkAppUri::from(self.zkapp_uri.clone()),
        })
    }
}

#[derive(Debug, Clone)]
pub enum AccountConfigError {
    MalformedCurrencyValue(String),
    MalformedKey(String),
    MalformedTokenId(String),
    MalformedReceiptChainHash(String),
    MalformedVotingFor(String),
    MalformedSlot(String),
    MalformedFp(String),
    ZkAppStateTooLong(Vec<String>),
    VerificationKeyParsingNotSupported,
    DelegateSetOnNonDefaultTokenAccount,
}

#[derive(Debug, Clone, Serialize)]
struct GlobalSlotSinceGenesis(u32);

impl<'de> Deserialize<'de> for GlobalSlotSinceGenesis {
    fn deserialize<D>(deserializer: D) -> Result<GlobalSlotSinceGenesis, D::Error>
    where
        D: Deserializer<'de>,
    {
        let err = "Global slot since genesis must consist of a tag and a number";
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Number(n) => {
                let slot = n.as_u64().ok_or(serde::de::Error::custom(err))? as u32;
                Ok(GlobalSlotSinceGenesis(slot))
            }
            Value::String(s) => {
                let slot = s
                    .parse::<u32>()
                    .map_err(|_| serde::de::Error::custom(err))?;
                Ok(GlobalSlotSinceGenesis(slot))
            }
            Value::Array(arr) => {
                let tag = arr
                    .first()
                    .and_then(Value::as_str)
                    .ok_or(serde::de::Error::custom(err))?;
                if tag != "Since_genesis" {
                    return Err(serde::de::Error::custom(err));
                };
                let s = arr
                    .get(1)
                    .and_then(Value::as_str)
                    .ok_or(serde::de::Error::custom(err))?;
                let slot = s
                    .parse::<u32>()
                    .map_err(|_| serde::de::Error::custom(err))?;
                Ok(GlobalSlotSinceGenesis(slot))
            }
            _ => Err(serde::de::Error::custom(err)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct GlobalSlotSpan(u32);

impl<'de> Deserialize<'de> for GlobalSlotSpan {
    fn deserialize<D>(deserializer: D) -> Result<GlobalSlotSpan, D::Error>
    where
        D: Deserializer<'de>,
    {
        let err = "Global slot span must consist of a tag and a number";
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Number(n) => {
                let slot = n.as_u64().ok_or(serde::de::Error::custom(err))? as u32;
                Ok(GlobalSlotSpan(slot))
            }
            Value::String(s) => {
                let slot = s
                    .parse::<u32>()
                    .map_err(|_| serde::de::Error::custom(err))?;
                Ok(GlobalSlotSpan(slot))
            }
            Value::Array(arr) => {
                let tag = arr
                    .first()
                    .and_then(Value::as_str)
                    .ok_or(serde::de::Error::custom(err))?;
                if tag != "Global_slot_span" {
                    return Err(serde::de::Error::custom(err));
                };
                let s = arr
                    .get(1)
                    .and_then(Value::as_str)
                    .ok_or(serde::de::Error::custom(err))?;
                let slot = s
                    .parse::<u32>()
                    .map_err(|_| serde::de::Error::custom(err))?;
                Ok(GlobalSlotSpan(slot))
            }
            _ => Err(serde::de::Error::custom(err)),
        }
    }
}

impl Display for AccountConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Account configuration error encountered in JSON config: "
        )?;
        match self {
            Self::MalformedCurrencyValue(c) => write!(f, "malformed currency value ('{}')", c),
            Self::MalformedKey(k) => write!(f, "malformed key ('{}')", k),
            Self::MalformedTokenId(tid) => write!(f, "malformed token id ('{}')", tid),
            Self::MalformedReceiptChainHash(rch) => {
                write!(f, "malformed receipt chain hash ('{}')", rch)
            }
            Self::MalformedVotingFor(vf) => write!(f, "malformed voting_for ('{}')", vf),
            Self::MalformedSlot(s) => write!(f, "malformed slot ('{}')", s),
            Self::MalformedFp(fp) => write!(f, "malformed field value ('{}')", fp),
            Self::ZkAppStateTooLong(app_state) => {
                write!(f, "zkapp app state too long ('{:?}')", app_state)
            }
            Self::VerificationKeyParsingNotSupported => {
                write!(f, "verification key parsing not supported yet!")
            }
            Self::DelegateSetOnNonDefaultTokenAccount => {
                write!(f, "delegate set on non-default token account")
            }
        }
    }
}

impl std::error::Error for AccountConfigError {}

use std::{borrow::Cow, fs::File, path::PathBuf};

use reqwest::Url;
use serde::Deserialize;

use ledger::{
    scan_state::currency::{Balance, Slot, SlotSpan},
    Account, BaseLedger, Database, Mask, Timing,
};

use mina_p2p_messages::{binprot::BinProtWrite, v2};
use mina_signer::CompressedPubKey;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    input: Option<PathBuf>,
    #[structopt(long)]
    url: Option<Url>,
    #[structopt(short, long)]
    output: PathBuf,
}

fn parse_account(mut a: serde_json::Value) -> anyhow::Result<Account> {
    let account_value = a
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("expected object"))?;

    let mut account = Account::empty();
    account.public_key = CompressedPubKey::from_address(
        account_value
            .remove("pk")
            .ok_or_else(|| anyhow::anyhow!("expected field `pk`"))?
            .clone()
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("`pk` must be string"))?,
    )?;
    if let Some(balance) = account_value.remove("balance") {
        let balance = balance
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("`balance` must be string"))?;
        let balance = if !balance.contains('.') {
            Cow::Owned(format!("{balance}.000000000"))
        } else {
            Cow::Borrowed(balance)
        };
        account.balance = Balance::of_mina_string_exn(&balance);
    }
    if let Some(delegate) = account_value
        .remove("delegate")
        .and_then(|a| a.as_str().map(ToOwned::to_owned))
    {
        account.delegate = Some(CompressedPubKey::from_address(&delegate)?);
    } else {
        account.delegate = Some(account.public_key.clone());
    }
    if let Some(timing) = account_value.remove("timing") {
        #[derive(Deserialize, Debug)]
        struct Timed {
            initial_minimum_balance: String,
            cliff_time: [String; 2],
            cliff_amount: String,
            vesting_period: [String; 2],
            vesting_increment: String,
        }

        let Timed {
            mut initial_minimum_balance,
            cliff_time,
            mut cliff_amount,
            vesting_period,
            mut vesting_increment,
        } = serde_json::from_value(timing.clone())?;

        if !initial_minimum_balance.contains('.') {
            initial_minimum_balance.extend(".000000000".chars());
        }
        if !cliff_amount.contains('.') {
            cliff_amount.extend(".000000000".chars());
        }
        if !vesting_increment.contains('.') {
            vesting_increment.extend(".000000000".chars());
        }

        account.timing = Timing::Timed {
            initial_minimum_balance: Balance::of_mina_string_exn(&initial_minimum_balance),
            cliff_time: Slot::from_u32(cliff_time[1].parse()?),
            cliff_amount: Balance::of_mina_string_exn(&cliff_amount).to_amount(),
            vesting_period: SlotSpan::from_u32(vesting_period[1].parse()?),
            vesting_increment: Balance::of_mina_string_exn(&vesting_increment).to_amount(),
        };
    }
    account_value.remove("sk");

    if !account_value.is_empty() {
        return Err(anyhow::anyhow!("the object contains unprocessed fields"));
    }

    Ok(account)
}

fn main() -> anyhow::Result<()> {
    let Args { input, url, output } = Args::from_args();

    let value = if let Some(input) = input {
        let ledger_file = File::open(&input)?;
        serde_json::from_reader::<_, serde_json::Value>(ledger_file)?
    } else if let Some(url) = url {
        reqwest::blocking::get(url)?.json()?
    } else {
        anyhow::bail!("must provide either `--input` or `--url`");
    };

    let list = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("expected object"))?
        .get("ledger")
        .ok_or_else(|| anyhow::anyhow!("expected field `ledger`"))?
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("expected object"))?
        .get("accounts")
        .ok_or_else(|| anyhow::anyhow!("expected field `accounts`"))?
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("expected array"))?;

    let genesis_winner_account = {
        let mut account = Account::empty();
        account.public_key = CompressedPubKey::from_address(
            "B62qiy32p8kAKnny8ZFwoMhYpBppM1DWVCqAPBYNcXnsAHhnfAAuXgg",
        )
        .expect("the constant is valid");

        account.balance = Balance::of_nanomina_int_exn(1000);
        account.delegate = Some(account.public_key.clone());
        account
    };

    let mut accounts = vec![genesis_winner_account];
    let mut mask = Mask::new_root(Database::create(35));

    for (n, item) in list.iter().enumerate() {
        accounts.push(
            parse_account(item.clone())
                .map_err(|err| anyhow::anyhow!("account: {n}, err: {err}"))?,
        );
    }

    for account in &accounts {
        let account_id = account.id();
        mask.get_or_create_account(account_id, account.clone())
            .expect("must not be full");
    }

    let root = mask.merkle_root();
    let top_hash = v2::LedgerHash::from_fp(root);
    println!("{top_hash}");

    let mut hashes = vec![];

    for (idx, hash) in mask.get_raw_inner_hashes() {
        let hash = v2::LedgerHash::from_fp(hash);
        hashes.push((idx, hash));
    }

    let mut output = File::create(&output)?;
    Some(top_hash).binprot_write(&mut output)?;
    hashes.binprot_write(&mut output)?;
    accounts.binprot_write(&mut output)?;

    Ok(())
}

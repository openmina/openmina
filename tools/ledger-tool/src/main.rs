use std::{
    fs::{self, File},
    path::PathBuf,
};

use reqwest::Url;

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

fn main() -> anyhow::Result<()> {
    let Args { input, url, output } = Args::from_args();

    let data = if let Some(input) = input {
        fs::read(input)?
    } else if let Some(url) = url {
        reqwest::blocking::get(url)?.bytes()?.to_vec()
    } else {
        anyhow::bail!("must provide either `--input` or `--url`");
    };

    let daemon_json = serde_json::from_slice::<node::daemon_json::DaemonJson>(&data)?;

    let prebuilt_config =
        node::transition_frontier::genesis::PrebuiltGenesisConfig::try_from(daemon_json)?;

    prebuilt_config.store(File::create(output)?)?;

    Ok(())
}

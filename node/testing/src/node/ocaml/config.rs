use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;

use node::account::AccountSecretKey;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcamlNodeTestingConfig {
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
    pub daemon_json: DaemonJson,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DaemonJson {
    // TODO(binier): have presets.
    Custom(String),
    InMem(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DaemonJsonGenConfig {
    Counts { whales: usize, fish: usize },
    DelegateTable(Vec<(u64, Vec<u64>)>),
}

#[derive(Debug, Clone)]
pub struct OcamlNodeConfig {
    /// Command for mina executable.
    pub executable: OcamlNodeExecutable,
    pub dir: temp_dir::TempDir,
    pub libp2p_keypair_i: usize,
    pub libp2p_port: u16,
    pub graphql_port: u16,
    pub client_port: u16,
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
    pub daemon_json: DaemonJson,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OcamlNodeExecutable {
    Installed(String),
    Docker(String),
    DockerDefault,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OcamlVrfOutput {
    pub vrf_output: String,
    pub vrf_output_fractional: f64,
    pub threshold_met: bool,
    pub public_key: String,
}

impl OcamlNodeConfig {
    /// Warning: All envs that needs to be set must be set here,
    /// otherwise it won't work for docker executable because env needs
    /// to be set from args.
    pub fn cmd<I, K, V>(&self, envs: I) -> Command
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        match &self.executable {
            OcamlNodeExecutable::Installed(program) => {
                let mut cmd = Command::new(program);
                cmd.envs(envs);
                cmd
            }
            OcamlNodeExecutable::Docker(tag) => self.docker_run_cmd(tag, envs),
            OcamlNodeExecutable::DockerDefault => {
                self.docker_run_cmd(OcamlNodeExecutable::DEFAULT_DOCKER_IMAGE, envs)
            }
        }
    }

    fn docker_run_cmd<I, K, V>(&self, tag: &str, envs: I) -> Command
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        let mut cmd = Command::new("docker");
        let dir_path = self.dir.path().display();

        let uid = std::env::var("$UID").unwrap_or_else(|_| "1000".to_owned());
        let container_name = OcamlNodeExecutable::docker_container_name(&self.dir);

        // set docker opts
        cmd.arg("run")
            .args(["--name".to_owned(), container_name])
            .args(["--network", "host"])
            .args(["--user".to_owned(), format!("{uid}:{uid}")])
            .args(["-v".to_owned(), format!("{dir_path}:{dir_path}")])
            // set workdir to `/tmp`, otherwise generating libp2p keys
            // using mina cmd might fail, if the user `$UID` doesn't
            // have a write permission in the default workdir.
            .args(["-w", "/tmp"]);

        // set docker container envs
        for (key, value) in envs {
            let arg: OsString = [key.as_ref(), value.as_ref()].join(OsStr::new("="));
            cmd.args(["-e".as_ref(), arg.as_os_str()]);
        }

        // set docker image
        cmd.arg(tag);

        cmd
    }
}

impl OcamlNodeExecutable {
    pub const DEFAULT_DOCKER_IMAGE: &'static str = "vladsimplestakingcom/mina-light:2.0.0rampup4";
    pub const DEFAULT_MINA_EXECUTABLE: &'static str = "mina";

    fn docker_container_name<'a>(tmp_dir: &temp_dir::TempDir) -> String {
        let path = tmp_dir.path().file_name().unwrap().to_str().unwrap();
        format!("openmina_testing_ocaml_{}", &path[1..])
    }

    /// Additional logic for killing the node.
    pub fn kill(&self, tmp_dir: &temp_dir::TempDir) {
        match self {
            OcamlNodeExecutable::Installed(_) => {}
            OcamlNodeExecutable::Docker(_) | OcamlNodeExecutable::DockerDefault => {
                // stop container.
                let mut cmd = Command::new("docker");
                let name = Self::docker_container_name(tmp_dir);
                cmd.args(["stop".to_owned(), name]);
                let _ = cmd.status();

                // remove container.
                let mut cmd = Command::new("docker");
                let name = Self::docker_container_name(tmp_dir);
                cmd.args(["rm".to_owned(), name]);
                let _ = cmd.status();
            }
        }
    }

    pub fn find_working() -> anyhow::Result<Self> {
        let program_name = Self::DEFAULT_MINA_EXECUTABLE;
        match Command::new(program_name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(_) => return Ok(Self::Installed(program_name.to_owned())),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => anyhow::bail!("'{program_name}' returned an error: {err}"),
            },
        };

        let mut cmd = Command::new("docker");

        let status = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .args(["pull", Self::DEFAULT_DOCKER_IMAGE])
            .status()
            .map_err(|err| anyhow::anyhow!("error pulling ocaml docker: {err}"))?;
        if !status.success() {
            anyhow::bail!("error status pulling ocaml node: {status:?}");
        }

        Ok(Self::DockerDefault)
    }
}

impl DaemonJson {
    pub fn load(
        mut add_account_sec_key: impl FnMut(AccountSecretKey),
        path: PathBuf,
        set_timestamp: Option<&str>,
    ) -> Self {
        let mut deamon_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();

        if let Some(time_str) = set_timestamp {
            deamon_json["genesis"]["genesis_state_timestamp"] = time_str.into();
        }

        deamon_json
            .get("ledger")
            .unwrap()
            .get("accounts")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .for_each(|val| {
                let sec_key_str = val.get("sk").unwrap().as_str().unwrap();
                add_account_sec_key(AccountSecretKey::from_str(sec_key_str).unwrap());
            });

        Self::InMem(deamon_json)
    }

    pub fn gen(
        add_account_sec_key: impl FnMut(AccountSecretKey),
        genesis_timestamp: &str,
        config: DaemonJsonGenConfig,
    ) -> Self {
        match config {
            DaemonJsonGenConfig::Counts { whales, fish } => {
                Self::gen_with_counts(add_account_sec_key, genesis_timestamp, whales, fish)
            }
            DaemonJsonGenConfig::DelegateTable(delegate_table) => Self::gen_with_delegate_table(
                add_account_sec_key,
                genesis_timestamp,
                delegate_table,
            ),
        }
    }

    pub fn gen_with_counts(
        add_account_sec_key: impl FnMut(AccountSecretKey),
        genesis_timestamp: &str,
        whales_n: usize,
        fish_n: usize,
    ) -> Self {
        let delegator_balance = |balance: u64| move |i| balance / i as u64;
        let whales = (0..whales_n).map(|i| {
            let balance = 8333_u64;
            let delegators = (1..=(i + 1) * 2).map(delegator_balance(50_000_000));
            (balance, delegators)
        });
        let fish = (0..fish_n).map(|i| {
            let balance = 6333_u64;
            let delegators = (1..=(i + 1) * 2).map(delegator_balance(5_000_000));
            (balance, delegators)
        });
        let delegate_table = whales.chain(fish);
        Self::gen_with_delegate_table(add_account_sec_key, genesis_timestamp, delegate_table)
    }

    pub fn gen_with_delegate_table(
        mut add_account_sec_key: impl FnMut(AccountSecretKey),
        genesis_timestamp: &str,
        delegate_table: impl IntoIterator<Item = (u64, impl IntoIterator<Item = u64>)>,
    ) -> Self {
        let gen_bp = |balance: u64| {
            let sec_key = AccountSecretKey::rand();
            let pub_key = sec_key.public_key();
            let account = serde_json::json!({
                "sk": sec_key.to_string(),
                "pk": pub_key.to_string(),
                "balance": format!("{balance}.000000000"),
                "delegate": pub_key.to_string(),
            });
            (sec_key, account)
        };
        let gen_account = |balance: u64, delegate: &str| {
            let (sec_key, mut account) = gen_bp(balance);
            account["delegate"] = delegate.into();
            (sec_key, account)
        };

        let all_accounts = delegate_table
            .into_iter()
            .flat_map(|(bp_balance, delegate_balances)| {
                let bp = gen_bp(bp_balance);
                let bp_pub_key = bp.0.public_key().to_string();
                let delegates = delegate_balances
                    .into_iter()
                    .map(move |balance| gen_account(balance, &bp_pub_key));
                std::iter::once(bp).chain(delegates)
            })
            .map(|(sec_key, account)| {
                add_account_sec_key(sec_key);
                account
            })
            .collect::<Vec<_>>();

        DaemonJson::InMem(serde_json::json!({
            "genesis": {
                "genesis_state_timestamp": genesis_timestamp,
            },
            "ledger": {
                "name": "custom",
                "accounts": all_accounts,
            },
        }))
    }
}

impl DaemonJsonGenConfig {
    pub fn block_producer_count(&self) -> usize {
        match self {
            Self::Counts { whales, fish } => whales + fish,
            Self::DelegateTable(bps) => bps.len(),
        }
    }
}

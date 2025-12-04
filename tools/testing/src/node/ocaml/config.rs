//! OCaml Node Configuration Module
//!
//! This module provides configuration structures and executable management
//! for OCaml Mina nodes in the testing framework. It supports multiple
//! execution modes including local binaries and Docker containers.
//!
//! # Key Components
//!
//! - [`OcamlNodeExecutable`] - Execution method selection (local/Docker)
//! - [`OcamlNodeTestingConfig`] - High-level node configuration
//! - [`OcamlNodeConfig`] - Low-level process configuration
//! - [`DaemonJson`] - Genesis configuration management
//!
//! # Executable Auto-Detection
//!
//! The module automatically detects the best available execution method:
//! 1. Local `mina` binary (preferred)
//! 2. Docker with default image (fallback)
//! 3. Custom Docker images (configurable)

use std::{
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};

use node::{
    account::AccountSecretKey,
    core::log::{info, system_time, warn},
    p2p::connection::outgoing::P2pConnectionOutgoingInitOpts,
};
use serde::{Deserialize, Serialize};

/// High-level configuration for OCaml node testing scenarios.
///
/// This struct provides the main configuration interface for creating
/// OCaml nodes in test scenarios, abstracting away low-level details
/// like port allocation and process management.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcamlNodeTestingConfig {
    /// List of initial peer connection targets
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
    /// Genesis ledger configuration (file path or in-memory)
    pub daemon_json: DaemonJson,
    /// Optional block producer secret key
    pub block_producer: Option<AccountSecretKey>,
}

impl Default for OcamlNodeTestingConfig {
    fn default() -> Self {
        Self {
            initial_peers: vec![],
            daemon_json: DaemonJson::Custom("/var/lib/coda/config_6929a7ec.json".to_owned()),
            block_producer: None,
        }
    }
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
    pub block_producer: Option<AccountSecretKey>,
}

/// OCaml node execution methods.
///
/// Supports multiple ways of running the OCaml Mina daemon,
/// from local binaries to Docker containers with automatic
/// detection and fallback behavior.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OcamlNodeExecutable {
    /// Use locally installed Mina binary
    ///
    /// # Arguments
    /// * `String` - Path to the mina executable
    ///
    /// # Example
    /// ```
    /// OcamlNodeExecutable::Installed("/usr/local/bin/mina".to_string())
    /// ```
    Installed(String),

    /// Use specific Docker image
    ///
    /// # Arguments
    /// * `String` - Docker image tag
    ///
    /// # Example
    /// ```
    /// OcamlNodeExecutable::Docker("minaprotocol/mina-daemon:3.0.0".to_string())
    /// ```
    Docker(String),

    /// Use default Docker image
    ///
    /// Falls back to the predefined default image when no local
    /// binary is available. See [`OcamlNodeExecutable::DEFAULT_DOCKER_IMAGE`] for the
    /// current default.
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
                info!(system_time(); "Using local Mina binary: {}", program);
                let mut cmd = Command::new(program);
                cmd.envs(envs);
                cmd
            }
            OcamlNodeExecutable::Docker(tag) => {
                info!(system_time(); "Using custom Docker image: {}", tag);
                self.docker_run_cmd(tag, envs)
            }
            OcamlNodeExecutable::DockerDefault => {
                info!(
                    system_time();
                    "Using default Docker image: {}",
                    OcamlNodeExecutable::DEFAULT_DOCKER_IMAGE
                );
                self.docker_run_cmd(OcamlNodeExecutable::DEFAULT_DOCKER_IMAGE, envs)
            }
        }
    }

    /// Create a Docker run command with proper configuration.
    ///
    /// Sets up a Docker container with appropriate networking, user mapping,
    /// volume mounts, and environment variables for running OCaml Mina daemon.
    ///
    /// # Arguments
    /// * `tag` - Docker image tag to use
    /// * `envs` - Environment variables to pass to the container
    ///
    /// # Docker Configuration
    /// - Uses host networking for P2P connectivity
    /// - Maps host user ID to avoid permission issues
    /// - Mounts node directory for persistent data
    /// - Sets working directory to `/tmp` for key generation
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

        info!(
            system_time();
            "Configuring Docker container: name={}, image={}, uid={}, mount={}",
            container_name,
            tag,
            uid,
            dir_path
        );

        // set docker opts
        cmd.arg("run")
            .args(["--name".to_owned(), container_name.clone()])
            .args(["--network", "host"])
            .args(["--user".to_owned(), format!("{uid}:{uid}")])
            .args(["-v".to_owned(), format!("{dir_path}:{dir_path}")])
            // set workdir to `/tmp`, otherwise generating libp2p keys
            // using mina cmd might fail, if the user `$UID` doesn't
            // have a write permission in the default workdir.
            .args(["-w", "/tmp"]);

        // set docker container envs
        let mut env_count = 0;
        for (key, value) in envs {
            let arg: OsString = [key.as_ref(), value.as_ref()].join(OsStr::new("="));
            cmd.args(["-e".as_ref(), arg.as_os_str()]);
            env_count += 1;
        }

        info!(system_time(); "Added {} environment variables to Docker container", env_count);

        // set docker image
        cmd.arg(tag);

        info!(system_time(); "Docker command configured for container: {}", container_name);
        cmd
    }
}

impl OcamlNodeExecutable {
    pub const DEFAULT_DOCKER_IMAGE: &'static str =
        "gcr.io/o1labs-192920/mina-daemon:3.3.0-alpha1-6929a7e-noble-devnet";
    pub const DEFAULT_MINA_EXECUTABLE: &'static str = "mina";

    fn docker_container_name(tmp_dir: &temp_dir::TempDir) -> String {
        let path = tmp_dir.path().file_name().unwrap().to_str().unwrap();
        format!("mina_testing_ocaml_{}", &path[1..])
    }

    /// Clean up resources when terminating an OCaml node.
    ///
    /// Handles cleanup logic specific to the execution method:
    /// - Local binaries: No additional cleanup needed
    /// - Docker containers: Stop and remove the container
    ///
    /// # Arguments
    /// * `tmp_dir` - Temporary directory used by the node
    pub fn kill(&self, tmp_dir: &temp_dir::TempDir) {
        match self {
            OcamlNodeExecutable::Installed(program) => {
                info!(system_time(); "No additional cleanup needed for local binary: {}", program);
            }
            OcamlNodeExecutable::Docker(_) | OcamlNodeExecutable::DockerDefault => {
                let name = Self::docker_container_name(tmp_dir);
                let image_info = match self {
                    OcamlNodeExecutable::Docker(img) => img.clone(),
                    OcamlNodeExecutable::DockerDefault => Self::DEFAULT_DOCKER_IMAGE.to_string(),
                    _ => unreachable!(),
                };

                info!(
                    system_time();
                    "Cleaning up Docker container: {} (image: {})",
                    name,
                    image_info
                );

                // stop container.
                info!(system_time(); "Stopping Docker container: {}", name);
                let mut cmd = Command::new("docker");
                cmd.args(["stop".to_owned(), name.clone()]);
                match cmd.status() {
                    Ok(status) if status.success() => {
                        info!(system_time(); "Successfully stopped Docker container: {}", name);
                    }
                    Ok(status) => {
                        warn!(
                            system_time();
                            "Docker stop command failed for container {}: exit code {:?}",
                            name,
                            status.code()
                        );
                    }
                    Err(e) => {
                        warn!(system_time(); "Failed to stop Docker container {}: {}", name, e);
                    }
                }

                // remove container.
                info!(system_time(); "Removing Docker container: {}", name);
                let mut cmd = Command::new("docker");
                cmd.args(["rm".to_owned(), name.clone()]);
                match cmd.status() {
                    Ok(status) if status.success() => {
                        info!(system_time(); "Successfully removed Docker container: {}", name);
                    }
                    Ok(status) => {
                        warn!(
                            system_time();
                            "Docker rm command failed for container {}: exit code {:?}",
                            name,
                            status.code()
                        );
                    }
                    Err(e) => {
                        warn!(system_time(); "Failed to remove Docker container {}: {}", name, e);
                    }
                }
            }
        }
    }

    /// Automatically detect and return the best available OCaml executable.
    ///
    /// This method implements the auto-detection strategy:
    /// 1. First, attempt to use locally installed `mina` binary
    /// 2. If not found, fall back to Docker with default image
    /// 3. Automatically pull the Docker image if needed
    ///
    /// # Returns
    /// * `Ok(OcamlNodeExecutable)` - Best available execution method
    /// * `Err(anyhow::Error)` - No usable execution method found
    ///
    /// # Docker Fallback
    /// When falling back to Docker, this method will automatically
    /// pull the default image if not already present locally.
    pub fn find_working() -> anyhow::Result<Self> {
        let program_name = Self::DEFAULT_MINA_EXECUTABLE;
        info!(system_time(); "Attempting to find local Mina binary: {}", program_name);

        match Command::new(program_name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(_) => {
                info!(system_time(); "Found working local Mina binary: {}", program_name);
                return Ok(Self::Installed(program_name.to_owned()));
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    info!(system_time(); "Local Mina binary not found, falling back to Docker");
                }
                _ => anyhow::bail!("'{program_name}' returned an error: {err}"),
            },
        };

        info!(
            system_time();
            "Pulling default Docker image: {}",
            Self::DEFAULT_DOCKER_IMAGE
        );
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

        info!(system_time(); "Successfully pulled Docker image, using DockerDefault");
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

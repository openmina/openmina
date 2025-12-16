use anyhow::Context;
use ledger::proofs::provers::BlockProver;
use mina_node_account::AccountPublicKey;
use mina_node_native::{archive::config::ArchiveStorageOptions, tracing, NodeBuilder};
use node::{
    account::AccountSecretKey,
    core::log::inner::Level,
    p2p::{connection::outgoing::P2pConnectionOutgoingInitOpts, identity::SecretKey},
    service::Recorder,
    snark::{BlockVerifier, TransactionVerifier},
    transition_frontier::genesis::GenesisConfig,
    SnarkerStrategy,
};
use reqwest::Url;
use std::{fs::File, path::PathBuf, sync::Arc};

/// Mina node configuration and runtime options
///
/// This struct defines all available command-line parameters for running a Mina node.
/// The node can operate in different modes (basic node, block producer, archive node)
/// depending on the parameters provided.
///
/// # Basic Usage
///
/// ```bash
/// # Run a basic node on devnet
/// mina node --network devnet
///
/// # Run with custom ports and logging
/// mina node --network devnet --port 3001 --libp2p-port 8303 --verbosity debug
/// ```
///
/// # Block Producer Mode
///
/// ```bash
/// # Run as block producer
/// mina node --network devnet --producer-key /path/to/key --coinbase-receiver B62q...
/// ```
///
/// # Archive Node Mode
///
/// ```bash
/// # Run as archive node with local storage
/// mina node --network devnet --archive-local-storage
/// ```
#[derive(Debug, clap::Args)]
pub struct Node {
    /// Working directory for node data, logs, and configuration files
    ///
    /// Can be set via MINA_HOME environment variable.
    /// Defaults to ~/.mina
    #[arg(long, short = 'd', default_value = "~/.mina", env = "MINA_HOME")]
    pub work_dir: String,

    /// P2P networking secret key for node identity
    ///
    /// If not provided, a new key will be generated automatically.
    /// Can be set via MINA_P2P_SEC_KEY environment variable.
    #[arg(long, short = 's', env = "MINA_P2P_SEC_KEY")]
    pub p2p_secret_key: Option<SecretKey>,

    // warning, this overrides `MINA_P2P_SEC_KEY`
    /// Compatibility with OCaml Mina node
    #[arg(long)]
    pub libp2p_keypair: Option<String>,

    // warning, this overrides `MINA_P2P_SEC_KEY`
    /// Compatibility with OCaml Mina node
    #[arg(env = "MINA_LIBP2P_PASS")]
    pub libp2p_password: Option<String>,

    /// List of external addresses at which this node is accessible
    #[arg(long)]
    pub libp2p_external_ip: Vec<String>,

    /// HTTP server port for RPC API and web interface
    ///
    /// The node will serve its HTTP API and dashboard on this port.
    /// Default: 3000
    #[arg(long, short, env, default_value = "3000")]
    pub port: u16,

    /// LibP2P networking port for peer-to-peer communication
    ///
    /// This port is used for connecting to other nodes in the network.
    /// Default: 8302
    #[arg(long, env, default_value = "8302")]
    pub libp2p_port: u16,

    /// Logging verbosity level
    ///
    /// Controls the amount of logging output. Options in order of verbosity:
    /// - error: Only show errors
    /// - warn: Show warnings and errors
    /// - info: Show informational messages, warnings, and errors (default)
    /// - debug: Show debug information and all above
    /// - trace: Show all possible logging output
    #[arg(long, short, env, default_value = "info")]
    pub verbosity: Level,

    /// Disable filesystem logging
    #[arg(long, env = "MINA_DISABLE_FILESYSTEM_LOGGING", default_value_t = false)]
    pub disable_filesystem_logging: bool,

    /// Specify custom path for log files
    #[arg(long, env = "MINA_LOG_PATH", default_value = "$MINA_HOME")]
    pub log_path: String,

    /// Initial peers to connect to on startup
    ///
    /// Specify peer multiaddresses to connect to when the node starts.
    /// Can be used multiple times to add multiple peers.
    ///
    /// # Multiaddr Format
    ///
    /// Multiaddresses follow the format: `/protocol/address/protocol/port/protocol/peer_id`
    ///
    /// **IPv4 Example:**
    /// ```
    /// /ip4/192.168.1.100/tcp/8302/p2p/12D3KooWABCDEF1234567890abcdef...
    /// ```
    ///
    /// **IPv6 Example:**
    /// ```
    /// /ip6/2001:db8::1/tcp/8302/p2p/12D3KooWABCDEF1234567890abcdef...
    /// ```
    ///
    /// **DNS Example:**
    /// ```
    /// /dns4/node.example.com/tcp/8302/p2p/12D3KooWABCDEF1234567890abcdef...
    /// ```
    ///
    /// Where:
    /// - `ip4/ip6/dns4` specifies the address type
    /// - IP address or hostname
    /// - `tcp` protocol with port number (typically 8302 for Mina)
    /// - `p2p` protocol with the peer's public key identifier
    #[arg(long, short = 'P', alias = "peer")]
    pub peers: Vec<P2pConnectionOutgoingInitOpts>,

    /// File containing initial peers to connect to
    ///
    /// Each line should contain a peer's multiaddr following the format described above.
    ///
    /// **Example file content:**
    /// ```
    /// /ip4/192.168.1.100/tcp/8302/p2p/12D3KooWABCDEF1234567890abcdef...
    /// /ip4/10.0.0.50/tcp/8302/p2p/12D3KooWXYZ9876543210fedcba...
    /// /dns4/bootstrap.example.com/tcp/8302/p2p/12D3KooW123ABC...
    /// ```
    ///
    /// Empty lines and lines starting with `#` are ignored.
    #[arg(long, env)]
    pub peer_list_file: Option<PathBuf>,

    /// URL to fetch initial peers list from
    ///
    /// The URL should return a text file with one peer multiaddr per line,
    /// using the same format as described in `peer_list_file`.
    /// Useful for dynamic peer discovery from a central bootstrap service.
    ///
    /// **Example URL response:**
    /// ```
    /// /ip4/bootstrap1.example.com/tcp/8302/p2p/12D3KooW...
    /// /ip4/bootstrap2.example.com/tcp/8302/p2p/12D3KooX...
    /// ```
    #[arg(long, env)]
    pub peer_list_url: Option<Url>,

    /// Maximum number of peer connections to maintain
    ///
    /// The node will attempt to maintain up to this many connections
    /// to other peers in the network. Default: 100
    #[arg(long, default_value = "100")]
    pub max_peers: usize,

    /// Run the node in seed mode. No default peers will be added.
    #[arg(long, env)]
    pub seed: bool,

    /// Run Snark Worker.
    ///
    /// Pass snarker private key as an argument.
    #[arg(long, env, group = "snarker")]
    pub run_snarker: Option<AccountSecretKey>,

    /// Snark fee, in Mina
    #[arg(long, env, default_value_t = 1_000_000, requires = "snarker")]
    pub snarker_fee: u64,

    #[arg(long, env, default_value = "seq", requires = "snarker")]
    pub snarker_strategy: SnarkerStrategy,

    /// Enable block producer with this key file
    ///
    /// MINA_PRIVKEY_PASS must be set to decrypt the keyfile if it is password-protected
    #[arg(long, env, group = "producer")]
    pub producer_key: Option<PathBuf>,

    /// Password used to decrypt the producer key file.
    #[arg(env = "MINA_PRIVKEY_PASS", default_value = "")]
    pub producer_key_password: String,

    /// Address to send coinbase rewards to (if this node is producing blocks).
    /// If not provided, coinbase rewards will be sent to the producer
    /// of a block.
    ///
    /// Warning: If the key is from a zkApp account, the account's
    /// receive permission must be None.
    #[arg(long, requires = "producer")]
    pub coinbase_receiver: Option<AccountPublicKey>,

    /// Enable recording of node state and actions for debugging and replay
    ///
    /// Recording captures the node's state transitions and input actions,
    /// enabling deterministic replay for debugging and testing purposes.
    ///
    /// Available modes:
    /// - `none`: No recording (default)
    /// - `state-with-input-actions`: Records initial state and all input
    ///   actions to the `recorder/` directory within the working directory
    ///
    /// Recorded data can be replayed using the `mina replay` command to
    /// reproduce the exact sequence of state transitions for debugging.
    ///
    /// # Example
    ///
    /// ```bash
    /// # Record node execution
    /// mina node --network devnet --record state-with-input-actions
    ///
    /// # Replay recorded execution
    /// mina replay state-with-input-actions ~/.mina/recorder
    /// ```
    #[arg(long, default_value = "none", env)]
    pub record: String,

    /// Do not use peers discovery.
    #[arg(long)]
    pub no_peers_discovery: bool,

    /// Config JSON file to load at startup.
    // TODO: make this argument required.
    #[arg(short = 'c', long, env)]
    pub config: Option<PathBuf>,

    /// Enable local precomputed storage.
    ///
    /// This option requires the following environment variables to be set:
    /// - MINA_ARCHIVE_LOCAL_STORAGE_PATH (otherwise the path to the working directory will be used)
    #[arg(long, env)]
    pub archive_local_storage: bool,

    /// Enable archiver process.
    ///
    /// This requires the following environment variables to be set:
    /// - MINA_ARCHIVE_ADDRESS
    #[arg(long, env)]
    pub archive_archiver_process: bool,

    /// Enable GCP precomputed storage.
    ///
    /// This requires the following environment variables to be set:
    /// - GCP_CREDENTIALS_JSON
    /// - GCP_BUCKET_NAME
    ///
    #[arg(long, env)]
    pub archive_gcp_storage: bool,

    /// Enable AWS precomputed storage.
    ///
    /// This requires the following environment variables to be set:
    /// - AWS_ACCESS_KEY_ID
    /// - AWS_SECRET_ACCESS_KEY
    /// - AWS_SESSION_TOKEN
    /// - AWS_DEFAULT_REGION
    /// - MINA_AWS_BUCKET_NAME
    #[arg(long, env)]
    pub archive_aws_storage: bool,

    #[arg(long, env)]
    pub rng_seed: Option<String>,
}

impl Node {
    pub fn run(self) -> anyhow::Result<()> {
        let work_dir = shellexpand::full(&self.work_dir).unwrap().into_owned();

        let _guard = if !self.disable_filesystem_logging {
            let log_output_dir = if self.log_path == "$MINA_HOME" {
                work_dir.clone()
            } else {
                self.log_path.clone()
            };
            Some(tracing::initialize_with_filesystem_output(
                self.verbosity,
                log_output_dir.into(),
            ))
        } else {
            tracing::initialize(self.verbosity);
            None
        };

        rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get().max(2) - 1)
            .thread_name(|i| format!("mina_rayon_{i}"))
            .build_global()
            .context("failed to initialize threadpool")?;

        let (daemon_conf, genesis_conf) = match self.config {
            Some(config) => {
                let reader = File::open(config).context("config file {config:?}")?;
                let config: node::daemon_json::DaemonJson =
                    serde_json::from_reader(reader).context("config file {config:?}")?;
                (
                    config
                        .daemon
                        .clone()
                        .unwrap_or(node::daemon_json::Daemon::DEFAULT),
                    Arc::new(GenesisConfig::DaemonJson(Box::new(config))),
                )
            }
            None => (
                node::daemon_json::Daemon::DEFAULT,
                node::config::DEVNET_CONFIG.clone(),
            ),
        };

        let custom_rng_seed = match self.rng_seed {
            None => None,
            Some(v) => match hex::decode(v)
                .map_err(anyhow::Error::from)
                .and_then(|bytes| {
                    <[u8; 32]>::try_from(bytes.as_slice()).map_err(anyhow::Error::from)
                }) {
                Ok(v) => Some(v),
                Err(err) => {
                    node::core::error!(
                        node::core::log::system_time();
                        summary = "bad rng seed",
                        err = err.to_string(),
                    );
                    return Err(err);
                }
            },
        };
        let mut node_builder: NodeBuilder =
            NodeBuilder::new(custom_rng_seed, daemon_conf, genesis_conf);

        // let genesis_config = match self.config {
        //     Some(config_path) => GenesisConfig::DaemonJsonFile(config_path).into(),
        //     None => node::config::DEVNET_CONFIG.clone(),
        // };
        // let mut node_builder: NodeBuilder = NodeBuilder::new(None, genesis_config);

        if let Some(sec_key) = self.p2p_secret_key {
            node_builder.p2p_sec_key(sec_key);
        }

        // warning, this overrides `MINA_P2P_SEC_KEY`
        if let (Some(key_file), Some(password)) = (&self.libp2p_keypair, &self.libp2p_password) {
            match SecretKey::from_encrypted_file(key_file, password) {
                Ok(sk) => {
                    node_builder.p2p_sec_key(sk.clone());
                    node::core::info!(
                        node::core::log::system_time();
                        summary = "read sercret key from file",
                        file_name = key_file,
                        pk = sk.public_key().to_string(),
                    )
                }
                Err(err) => {
                    node::core::error!(
                        node::core::log::system_time();
                        summary = "failed to read secret key",
                        file_name = key_file,
                        err = err.to_string(),
                    );
                    return Err(err.into());
                }
            }
        } else if self.libp2p_keypair.is_some() && self.libp2p_password.is_none() {
            let error = "keyfile is specified, but `MINA_LIBP2P_PASS` is not set";
            node::core::error!(
                node::core::log::system_time();
                summary = error,
            );
            return Err(anyhow::anyhow!(error));
        }

        node_builder.p2p_libp2p_port(self.libp2p_port);

        node_builder.external_addrs(
            self.libp2p_external_ip
                .into_iter()
                .filter_map(|s| s.parse().ok()),
        );

        node_builder.p2p_max_peers(self.max_peers);
        self.seed.then(|| node_builder.p2p_seed_node());
        self.no_peers_discovery
            .then(|| node_builder.p2p_no_discovery());

        node_builder.initial_peers(self.peers);
        if let Some(path) = self.peer_list_file {
            node_builder.initial_peers_from_file(path)?;
        }
        if let Some(url) = self.peer_list_url {
            node_builder.initial_peers_from_url(url)?;
        }

        let block_verifier_index = BlockVerifier::make();
        let work_verifier_index = TransactionVerifier::make();
        node_builder
            .block_verifier_index(block_verifier_index.clone())
            .work_verifier_index(work_verifier_index.clone());

        if let Some(producer_key_path) = self.producer_key {
            let password = &self.producer_key_password;
            mina_core::thread::spawn(|| {
                node::core::info!(node::core::log::system_time(); summary = "loading provers index");
                BlockProver::make(Some(block_verifier_index), Some(work_verifier_index));
                node::core::info!(node::core::log::system_time(); summary = "loaded provers index");
            });
            node_builder.block_producer_from_file(producer_key_path, password, None)?;

            if let Some(pub_key) = self.coinbase_receiver {
                node_builder
                    .custom_coinbase_receiver(pub_key.into())
                    .unwrap();
            }
        }

        let archive_storage_options = ArchiveStorageOptions::from_iter(
            [
                (
                    self.archive_local_storage,
                    ArchiveStorageOptions::LOCAL_PRECOMPUTED_STORAGE,
                ),
                (
                    self.archive_archiver_process,
                    ArchiveStorageOptions::ARCHIVER_PROCESS,
                ),
                (
                    self.archive_gcp_storage,
                    ArchiveStorageOptions::GCP_PRECOMPUTED_STORAGE,
                ),
                (
                    self.archive_aws_storage,
                    ArchiveStorageOptions::AWS_PRECOMPUTED_STORAGE,
                ),
            ]
            .iter()
            .filter(|(enabled, _)| *enabled)
            .map(|(_, option)| option.clone()),
        );

        if archive_storage_options.is_enabled() {
            node::core::info!(
                summary = "Archive mode enabled",
                local_storage = archive_storage_options.uses_local_precomputed_storage(),
                archiver_process = archive_storage_options.uses_archiver_process(),
                gcp_storage = archive_storage_options.uses_gcp_precomputed_storage(),
                aws_storage = archive_storage_options.uses_aws_precomputed_storage(),
            );

            archive_storage_options
                .validate_env_vars()
                .map_err(|e| anyhow::anyhow!(e))?;

            node_builder.archive(archive_storage_options, work_dir.clone());
        }

        if let Some(sec_key) = self.run_snarker {
            node_builder.snarker(sec_key, self.snarker_fee, self.snarker_strategy);
        }

        mina_core::set_work_dir(work_dir.clone().into());

        node_builder
            .http_server(self.port)
            .gather_stats()
            .record(match self.record.trim() {
                "none" => Recorder::None,
                "state-with-input-actions" => Recorder::only_input_actions(work_dir),
                _ => panic!("unknown --record strategy"),
            });

        let mut node = node_builder.build().context("node build failed!")?;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .thread_stack_size(64 * 1024 * 1024)
            .build()
            .unwrap();

        runtime.block_on(node.run_forever());

        Ok(())
    }
}

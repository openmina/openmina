use mina_p2p_messages::v2::{self};
use node::core::{channels::mpsc, thread};
use node::ledger::write::BlockApplyResult;
use std::env;
use std::io::Write;

use mina_p2p_messages::v2::PrecomputedBlock;
use openmina_core::NetworkConfig;
use reqwest::Url;
use std::net::SocketAddr;
use std::{fs::File, path::Path};

use super::NodeService;

pub mod aws;
pub mod config;
pub mod gcp;
#[cfg(not(target_arch = "wasm32"))]
pub mod rpc;

use config::ArchiveStorageOptions;

const ARCHIVE_SEND_RETRIES: u8 = 5;
const MAX_EVENT_COUNT: u64 = 100;
const RETRY_INTERVAL_MS: u64 = 1000;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Environment variable {0} is not set")]
    EnvironmentVariableNotSet(String),
    #[error("Failed to upload block to AWS: {0}")]
    UploadError(String),
}

pub struct ArchiveService {
    archive_sender: mpsc::UnboundedSender<BlockApplyResult>,
}

struct ArchiveServiceClients {
    archiver_address: Option<SocketAddr>,
    aws_client: Option<aws::ArchiveAWSClient>,
    gcp_client: Option<gcp::ArchiveGCPClient>,
    local_path: Option<String>,
}

impl ArchiveServiceClients {
    async fn new(options: &ArchiveStorageOptions, work_dir: String) -> Result<Self, Error> {
        let aws_client = if options.uses_aws_precomputed_storage() {
            let client = aws::ArchiveAWSClient::new().await?;
            Some(client)
        } else {
            None
        };

        let gcp_client = if options.uses_gcp_precomputed_storage() {
            let client = gcp::ArchiveGCPClient::new().await?;
            Some(client)
        } else {
            None
        };

        let local_path = if options.uses_local_precomputed_storage() {
            let env_path = env::var("OPENMINA_LOCAL_PRECOMPUTED_STORAGE_PATH");
            let default = format!("{}/archive-precomputed", work_dir);
            Some(env_path.unwrap_or(default))
        } else {
            None
        };

        let archiver_address = if options.uses_archiver_process() {
            let address = std::env::var("OPENMINA_ARCHIVE_ADDRESS")
                .expect("OPENMINA_ARCHIVE_ADDRESS is not set");
            let address = Url::parse(&address).expect("Invalid URL");

            // Convert URL to SocketAddr
            let socket_addrs = address.socket_addrs(|| None).expect("Invalid URL");

            let socket_addr = socket_addrs.first().expect("No socket address found");

            Some(*socket_addr)
        } else {
            None
        };

        Ok(Self {
            archiver_address,
            aws_client,
            gcp_client,
            local_path,
        })
    }

    pub async fn send_block(&self, breadcrumb: BlockApplyResult, options: &ArchiveStorageOptions) {
        if options.uses_archiver_process() {
            if let Some(socket_addr) = self.archiver_address {
                Self::handle_archiver_process(&breadcrumb, &socket_addr).await;
            } else {
                node::core::warn!(summary = "Archiver address not set");
            }
        }

        if options.requires_precomputed_block() {
            let network_name = NetworkConfig::global().name;
            let height = breadcrumb.block.height();
            let state_hash = breadcrumb.block.hash();

            let key = format!("{network_name}-{height}-{state_hash}.json");

            node::core::info!(
                summary = "Uploading precomputed block to archive",
                key = key.clone()
            );

            let precomputed_block: PrecomputedBlock = if let Ok(precomputed_block) =
                breadcrumb.try_into()
            {
                precomputed_block
            } else {
                node::core::warn!(summary = "Failed to convert breadcrumb to precomputed block");
                return;
            };

            let data = serde_json::to_vec(&precomputed_block).unwrap();

            if options.uses_local_precomputed_storage() {
                // TODO(adonagy): Cleanup the unwraps (fn that returns a Result + log the error)
                if let Some(path) = &self.local_path {
                    let file_path = Path::new(path).join(key.clone());
                    let mut file = File::create(file_path).unwrap();
                    file.write_all(&data).unwrap();
                } else {
                    node::core::warn!(summary = "Local precomputed storage path not set");
                }
            }

            if options.uses_gcp_precomputed_storage() {
                if let Some(client) = &self.gcp_client {
                    if let Err(e) = client.upload_block(&key, &data).await {
                        node::core::warn!(
                            summary = "Failed to upload precomputed block to GCP",
                            error = e.to_string()
                        );
                    }
                } else {
                    node::core::warn!(summary = "GCP client not initialized");
                }
            }
            if options.uses_aws_precomputed_storage() {
                if let Some(client) = &self.aws_client {
                    if let Err(e) = client.upload_block(&key, &data).await {
                        node::core::warn!(
                            summary = "Failed to upload precomputed block to AWS",
                            error = e.to_string()
                        );
                    }
                } else {
                    node::core::warn!(summary = "AWS client not initialized");
                }
            }
        }
    }

    async fn handle_archiver_process(breadcrumb: &BlockApplyResult, socket_addr: &SocketAddr) {
        let mut retries = ARCHIVE_SEND_RETRIES;

        let archive_transition_frontier_diff: v2::ArchiveTransitionFronntierDiff =
            breadcrumb.clone().try_into().unwrap();

        while retries > 0 {
            match rpc::send_diff(
                *socket_addr,
                v2::ArchiveRpc::SendDiff(archive_transition_frontier_diff.clone()),
            ) {
                Ok(result) => {
                    if result.should_retry() {
                        node::core::warn!(
                            summary = "Archive suddenly closed connection, retrying..."
                        );
                        retries -= 1;
                        tokio::time::sleep(std::time::Duration::from_millis(RETRY_INTERVAL_MS))
                            .await;
                    } else {
                        node::core::warn!(summary = "Successfully sent diff to archive");
                        break;
                    }
                }
                Err(e) => {
                    node::core::warn!(
                        summary = "Failed sending diff to archive",
                        error = e.to_string(),
                        retries = retries
                    );
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_millis(RETRY_INTERVAL_MS)).await;
                }
            }
        }
    }
}

impl ArchiveService {
    fn new(archive_sender: mpsc::UnboundedSender<BlockApplyResult>) -> Self {
        Self { archive_sender }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn run(
        mut archive_receiver: mpsc::UnboundedReceiver<BlockApplyResult>,
        options: ArchiveStorageOptions,
        work_dir: String,
    ) {
        let clients = if let Ok(clients) = ArchiveServiceClients::new(&options, work_dir).await {
            clients
        } else {
            node::core::error!(summary = "Failed to initialize archive service clients");
            return;
        };

        while let Some(breadcrumb) = archive_receiver.recv().await {
            clients.send_block(breadcrumb, &options).await;
        }
    }

    // Note: Placeholder for the wasm implementation, if we decide to include an archive mode in the future
    #[cfg(target_arch = "wasm32")]
    fn run(
        mut archive_receiver: mpsc::UnboundedReceiver<ArchiveTransitionFronntierDiff>,
        address: SocketAddr,
        options: ArchiveStorageOptions,
    ) {
        unimplemented!()
    }

    pub fn start(options: ArchiveStorageOptions, work_dir: String) -> Self {
        let (archive_sender, archive_receiver) = mpsc::unbounded_channel::<BlockApplyResult>();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        thread::Builder::new()
            .name("openmina_archive".to_owned())
            .spawn(move || {
                runtime.block_on(Self::run(archive_receiver, options, work_dir));
            })
            .unwrap();

        Self::new(archive_sender)
    }
}

impl node::transition_frontier::archive::archive_service::ArchiveService for NodeService {
    fn send_to_archive(&mut self, data: BlockApplyResult) {
        if let Some(archive) = self.archive.as_mut() {
            if let Err(e) = archive.archive_sender.send(data) {
                node::core::warn!(
                    summary = "Failed sending diff to archive service",
                    error = e.to_string()
                );
            }
        }
    }
}

// Note: Placeholder for the wasm implementation, if we decide to include an archive mode in the future
#[cfg(target_arch = "wasm32")]
mod rpc {}

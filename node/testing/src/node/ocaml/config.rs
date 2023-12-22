use std::process::Command;

use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcamlNodeTestingConfig {
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
    pub daemon_json: DaemonJson,
    pub daemon_json_update_timestamp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DaemonJson {
    // TODO(binier): have presets.
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct OcamlNodeConfig {
    /// Command for mina executable.
    pub executable: OcamlNodeExecutable,
    pub dir: temp_dir::TempDir,
    pub libp2p_port: u16,
    pub graphql_port: u16,
    pub client_port: u16,
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
    pub daemon_json: DaemonJson,
    pub daemon_json_update_timestamp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OcamlNodeExecutable {
    Installed(String),
    Docker(Option<String>),
}

impl OcamlNodeConfig {
    pub const DEFAULT_DOCKER_TAG: &'static str = "openmina/testing/mina";

    pub fn cmd(&self) -> Command {
        match &self.executable {
            OcamlNodeExecutable::Installed(program) => Command::new(program),
            OcamlNodeExecutable::Docker(tag) => {
                let tag = tag
                    .as_ref()
                    .map(String::as_str)
                    .unwrap_or(Self::DEFAULT_DOCKER_TAG);
                self.docker_run_cmd(tag)
            }
        }
    }

    fn docker_run_cmd(&self, tag: &str) -> Command {
        let mut cmd = Command::new("docker");
        let dir_path = self.dir.path().display();
        cmd.arg("run")
            .args(["--network", "host"])
            .args(["-v".to_owned(), format!("{dir_path}:{dir_path}")])
            .arg(tag);
        cmd
    }
}

impl Default for OcamlNodeExecutable {
    fn default() -> Self {
        Self::Installed("mina".to_owned())
    }
}

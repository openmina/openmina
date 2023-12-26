use std::ffi::{OsStr, OsString};
use std::process::{Command, Stdio};

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
    Docker(String),
    DockerDefault,
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

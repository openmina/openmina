mod config;
pub use config::*;
use node::p2p::{
    PeerId, common::{P2pGenericAddrs, P2pGenericPeer}, libp2p::P2pLibP2pAddr,
};

use std::{
    path::{Path, PathBuf},
    process::Child,
    time::Duration,
};

use serde::{Deserialize, Serialize};

pub struct OcamlNode {
    child: Child,
    pub libp2p_port: u16,
    pub graphql_port: u16,
    peer_id: libp2p::PeerId,
    #[allow(dead_code)]
    temp_dir: temp_dir::TempDir,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OcamlStep {
    /// Wait till ocaml node is ready.
    ///
    /// Right now it simply waits till p2p port is ready. Is this enough?
    WaitReady {
        timeout: Duration,
    },
    Kill,
}

impl OcamlNode {
    pub fn start(config: OcamlNodeConfig) -> anyhow::Result<Self> {
        let dir = config.dir.path();
        let config_dir = dir.join(".config");
        let daemon_json_path = config_dir.join("daemon.json");

        std::fs::create_dir_all(&config_dir)
            .map_err(|err| anyhow::anyhow!("failed to create config dir: {err}"))?;

        let peer_id = match Self::read_peer_id(dir) {
            Ok(v) => v,
            Err(_) => Self::generate_libp2p_keypair(&config, dir).map_err(|err| {
                anyhow::anyhow!("failed to generate libp2p keys for ocaml node. err: {err}")
            })?,
        };
        let peer_id = peer_id.parse()?;

        if config.daemon_json_update_timestamp {
            todo!()
        } else {
            match &config.daemon_json {
                DaemonJson::Custom(path) => {
                    std::fs::copy(path, &daemon_json_path).map_err(|err| {
                        anyhow::anyhow!(
                            "failed to copy daemon_json from: '{}', to: '{}'; error: {}",
                            path,
                            daemon_json_path.display(),
                            err
                        )
                    })?;
                }
            }
        }

        let mut cmd = config.cmd([("MINA_LIBP2P_PASS", "")]);

        cmd.arg("daemon");
        cmd.arg("--config-dir").arg(&config_dir);
        cmd.arg("--libp2p-keypair").arg(&Self::privkey_path(dir));
        cmd.args(["--external-ip", "127.0.0.1"])
            .args(["--external-port", &config.libp2p_port.to_string()])
            .args(["--client-port", &config.client_port.to_string()])
            .args(["--rest-port", &config.graphql_port.to_string()]);

        let is_seed = config.initial_peers.is_empty();
        for peer in config.initial_peers {
            cmd.args(["--peer", &peer.to_string()]);
        }
        if is_seed {
            cmd.arg("--seed");
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("no stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("no stderr"))?;

        let prefix = format!("[localhost:{}] ", config.libp2p_port);
        let prefix2 = prefix.clone();
        std::thread::spawn(move || {
            if let Err(_) = Self::read_stream(stdout, std::io::stdout(), &prefix) {}
        });
        std::thread::spawn(move || {
            if let Err(_) = Self::read_stream(stderr, std::io::stderr(), &prefix2) {}
        });

        Ok(Self {
            child,
            libp2p_port: config.libp2p_port,
            graphql_port: config.graphql_port,
            peer_id,
            temp_dir: config.dir,
        })
    }

    pub fn dial_addr(&self) -> P2pGenericAddrs {
        P2pGenericAddrs::LibP2p(vec![
            P2pLibP2pAddr {
                host: [127, 0, 0, 1].into(),
                port: self.libp2p_port,
            }
        ])
    }

    pub fn to_peers<T: FromIterator<P2pGenericPeer>>(&self) -> T {
        let peer_id = self.peer_id();
        self.dial_addr().to_generic_peers(&peer_id)
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id.into()
    }

    pub async fn exec(&mut self, step: OcamlStep) -> anyhow::Result<bool> {
        Ok(match step {
            OcamlStep::WaitReady { timeout } => {
                self.wait_for_p2p(timeout).await?;
                true
            }
            OcamlStep::Kill => {
                self.kill()?;
                true
            }
        })
    }

    fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }

    const PRIVKEY_PATH: &'static str = ".libp2p/key";

    fn privkey_path(dir: &Path) -> PathBuf {
        dir.join(Self::PRIVKEY_PATH)
    }

    fn read_peer_id(dir: &Path) -> anyhow::Result<String> {
        Ok(
            std::fs::read_to_string(Self::privkey_path(dir).with_extension("peerid"))?
                .trim()
                .into(),
        )
    }

    fn generate_libp2p_keypair(config: &OcamlNodeConfig, dir: &Path) -> anyhow::Result<String> {
        let mut child = config
            .cmd([("MINA_LIBP2P_PASS", ""), ("UMASK", "0700")])
            .args(["libp2p", "generate-keypair", "--privkey-path"])
            .arg(Self::privkey_path(dir))
            .spawn()?;
        if child.wait()?.success() {
            let peer_id = Self::read_peer_id(dir)?;
            Ok(peer_id)
        } else {
            anyhow::bail!("error generating keypair");
        }
    }

    fn read_stream<R: std::io::Read, W: std::io::Write>(
        from: R,
        mut to: W,
        prefix: &str,
    ) -> std::io::Result<()> {
        let mut buf = std::io::BufReader::new(from);
        let mut line = String::with_capacity(256);
        while std::io::BufRead::read_line(&mut buf, &mut line)? > 0 {
            to.write_all(prefix.as_bytes())?;
            to.write_all(line.as_bytes())?;
            line.clear();
        }
        Ok(())
    }

    fn graphql_addr(&self) -> String {
        format!("http://127.0.0.1:{}/graphql", self.graphql_port)
    }

    // TODO(binier): shouldn't be publically accessible.
    //
    // Only `exec` function should be exposed and instead of this, we
    // should have a step to query graphql and assert response as a part
    // of that step.
    pub fn grapql_query(&self, query: &str) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(self.graphql_addr())
            .json(&{
                serde_json::json!({
                    "query": query
                })
            })
            .send()?;

        Ok(response.json()?)
    }

    async fn wait_for_p2p(&self, timeout: Duration) -> anyhow::Result<()> {
        let port = self.libp2p_port;
        let timeout_fut = tokio::time::sleep(timeout);
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        let probe = tokio::task::spawn(async move {
            loop {
                interval.tick().await;
                match tokio::net::TcpStream::connect(("127.0.0.1", port)).await {
                    Ok(_) => return,
                    Err(_) => {}
                }
            }
        });
        tokio::select! {
            _ = timeout_fut => anyhow::bail!("waiting for ocaml node's p2p port to be ready timed out! timeout: {}ms", timeout.as_millis()),
            _ = probe => Ok(()),
        }
    }
}

impl Drop for OcamlNode {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Err(err) => {
                eprintln!("error getting status from OCaml node: {err}");
            }
            Ok(None) => {
                if let Err(err) = self.child.kill() {
                    eprintln!("error killing OCaml node: {err}");
                } else if let Err(err) = self.child.wait() {
                    eprintln!("error getting status from OCaml node: {err}");
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[test]
fn run_ocaml() {
    use std::io::{BufRead, BufReader};

    use crate::node::DaemonJson;

    let mut node = OcamlNode::start(OcamlNodeConfig {
        executable: OcamlNodeExecutable::find_working().unwrap(),
        dir: temp_dir::TempDir::new().unwrap(),
        libp2p_port: 8302,
        graphql_port: 3086,
        client_port: 8301,
        initial_peers: Vec::new(),
        daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
        daemon_json_update_timestamp: false,
    })
    .unwrap();
    let stdout = node.child.stdout.take().unwrap();
    std::thread::spawn(move || {
        for line in BufRead::lines(BufReader::new(stdout)) {
            println!("{}", line.unwrap());
        }
    });

    node.child.wait().unwrap();
}

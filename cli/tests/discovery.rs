use std::{
    ffi::OsStr,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command},
    thread::JoinHandle,
    time::Duration,
};

use ledger::Instant;
use node::p2p::{
    connection::outgoing::P2pConnectionOutgoingInitOpts, identity::SecretKey, P2pPeerStatus,
};

#[test]
#[ignore = "should be run manually"]
fn dicovery_seed_rust_ocaml() {
    let d = temp_dir::TempDir::new().unwrap();
    let seed = run_seed(&d.path().join("seed"));
    assert_p2p_ready(&seed, Duration::from_secs(5 * 60));

    let rust = run_rust_node(seed.peer_id());
    assert_p2p_ready(&rust, Duration::from_secs(1 * 60));

    assert_peers(&rust, &seed, Duration::from_secs(60));

    let ocaml = run_ocaml_node(&d.path().join("ocaml"), seed.peer_id());
    assert_p2p_ready(&ocaml, Duration::from_secs(5 * 60));

    assert_peers(&ocaml, &rust, Duration::from_secs(60));
}

#[test]
#[ignore = "should be run manually"]
fn dicovery_rust_seed_ocaml() {
    let d = temp_dir::TempDir::new().unwrap();
    let seed_dir = d.path().join("seed");
    let seed_peer_id = OCamlNode::generate_libp2p_keypair(&seed_dir).unwrap();

    let rust = run_rust_node(&seed_peer_id);
    assert_p2p_ready(&rust, Duration::from_secs(2 * 60));

    let seed = run_seed(&seed_dir);
    assert_p2p_ready(&seed, Duration::from_secs(5 * 60));

    assert_peers(&rust, &seed, Duration::from_secs(60));

    let ocaml = run_ocaml_node(&d.path().join("ocaml"), &seed_peer_id);
    assert_p2p_ready(&ocaml, Duration::from_secs(5 * 60));

    assert_peers(&ocaml, &rust, Duration::from_secs(60));
}

const SEED_ADDRESS: &str = "127.0.0.1:8302";
const OCAML_ADDRESS: &str = "127.0.0.1:18302";
const RUST_ADDRESS: &str = "127.0.0.1:28302";

fn run_seed(dir: &Path) -> OCamlNode {
    OCamlNode::run::<_, String>(SEED_ADDRESS.parse().unwrap(), dir, true, []).unwrap()
}

fn seed_multiaddr(peer_id: &str) -> String {
    let socket_addr = SEED_ADDRESS.parse::<SocketAddr>().unwrap();
    format!(
        "/ip4/{}/tcp/{}/p2p/{}",
        socket_addr.ip(),
        socket_addr.port(),
        peer_id,
    )
}

fn run_ocaml_node(dir: &Path, seed_peer_id: &str) -> OCamlNode {
    OCamlNode::run(
        OCAML_ADDRESS.parse().unwrap(),
        dir,
        false,
        [seed_multiaddr(seed_peer_id)],
    )
    .unwrap()
}

fn run_rust_node(seed_peer_id: &str) -> RustNode {
    RustNode::run(
        RUST_ADDRESS.parse().unwrap(),
        vec![seed_multiaddr(seed_peer_id)],
    )
}

struct OCamlNode {
    process: Child,
    address: SocketAddr,
    graphql_port: u16,
    peer_id: String,
}

const PEERS_QUERY: &str = r#"query {
  getPeers {
    host
    libp2pPort
    peerId
  }
}"#;

fn query_body(query: &str) -> serde_json::Value {
    serde_json::json!({
        "query": query
    })
}

impl OCamlNode {
    const PRIVKEY_PATH: &str = "libp2p";

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

    fn generate_libp2p_keypair(dir: &Path) -> anyhow::Result<String> {
        let mut child = Command::new("mina")
            .args(["libp2p", "generate-keypair", "--privkey-path"])
            .arg(Self::privkey_path(dir))
            .env("MINA_LIBP2P_PASS", "")
            .env("UMASK", "0700")
            .spawn()?;
        if child.wait()?.success() {
            let peer_id = Self::read_peer_id(dir)?;
            Ok(peer_id)
        } else {
            anyhow::bail!("error generating keypair");
        }
    }

    fn run<I: IntoIterator<Item = S>, S: AsRef<str>>(
        address: SocketAddr,
        dir: &Path,
        is_seed: bool,
        peers: I,
    ) -> anyhow::Result<OCamlNode> {
        let peer_id = match Self::read_peer_id(dir) {
            Ok(v) => v,
            Err(_) => OCamlNode::generate_libp2p_keypair(dir)?,
        };
        eprintln!(">>> using peer_id {peer_id}");

        let mut command = Command::new("mina");
        let client_port = address.port() - 1;
        let graphql_port = address.port() + 1;
        command
            .arg("daemon")
            .args([
                OsStr::new("--libp2p-keypair"),
                Self::privkey_path(dir).as_os_str(),
            ])
            .args(["--external-ip", &address.ip().to_string()])
            .args(["--external-port", &address.port().to_string()])
            .args(["--client-port", &client_port.to_string()])
            .args(["--rest-port", &graphql_port.to_string()])
            // .args(["--proof-level", "none"])
            .args(["--config-file", "/var/lib/coda/berkeley.json"])
            .args([OsStr::new("--config-dir"), dir.as_os_str()])
            .env("MINA_LIBP2P_PASS", "");
        if is_seed {
            command.arg("--seed");
        }
        for peer in peers {
            command.args(["--peer", peer.as_ref()]);
        }
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = command.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("no stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("no stderr"))?;

        let prefix = format!("[{address}] ");
        let prefix2 = prefix.clone();
        std::thread::spawn(move || {
            if let Err(_) = Self::read_stream(stdout, std::io::stdout(), &prefix) {}
        });
        std::thread::spawn(move || {
            if let Err(_) = Self::read_stream(stderr, std::io::stderr(), &prefix2) {}
        });

        Ok(OCamlNode {
            process: child,
            address,
            graphql_port,
            peer_id,
        })
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
        let mut address = self.address.clone();
        address.set_port(self.graphql_port);
        format!("http://{address}/graphql")
    }

    fn grapql_query(&self, query: &str) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(self.graphql_addr())
            .json(&query_body(query))
            .send()?;

        Ok(response.json()?)
    }

    fn has_peer_json(peerid: &str, data: serde_json::Value) -> Option<bool> {
        for elt in data.as_object()?.get("data")?.get("getPeers")?.as_array()? {
            let elt = elt.as_object()?;
            let host = elt.get("host")?.as_str()?;
            let port = elt.get("libp2pPort")?.as_i64()?;
            let peer_address = format!("{host}:{port}").parse::<SocketAddr>().unwrap();
            eprintln!(">>> peer address: {peer_address}");
            if peerid != elt.get("peerId")?.as_str()? {
                continue;
            }
            return Some(true);
        }
        Some(false)
    }
}

impl MinaNode for OCamlNode {
    fn address(&self) -> &SocketAddr {
        &self.address
    }

    fn peer_id(&self) -> &str {
        &self.peer_id
    }

    fn has_peer(&self, peerid: &str) -> anyhow::Result<bool> {
        let json = self.grapql_query(PEERS_QUERY)?;
        let has_peer = Self::has_peer_json(peerid, json).expect("invalid json");
        Ok(has_peer)
    }
}

impl Drop for OCamlNode {
    fn drop(&mut self) {
        self.process.kill().unwrap();
        self.process.wait().unwrap();
    }
}

struct RustNode {
    _handle: JoinHandle<Result<(), String>>,
    address: SocketAddr,
    http_port: u16,
    peer_id: String,
}

impl RustNode {
    fn run<I: IntoIterator<Item = S>, S: AsRef<str>>(address: SocketAddr, peers: I) -> RustNode {
        let http_port = address.port() + 1;
        let secret_key = SecretKey::rand();
        let peer_id = secret_key.public_key().peer_id().to_libp2p_string();
        let command = cli::commands::node::Node {
            work_dir: "~/.openmina".into(),
            p2p_secret_key: Some(secret_key),
            port: http_port,
            libp2p_port: address.port(),
            verbosity: "debug".parse().unwrap(),
            peers: peers
                .into_iter()
                .map(|peer| peer.as_ref().parse().unwrap())
                .collect(),
            run_snarker: None,
            snarker_fee: 0,
            snarker_strategy: node::SnarkerStrategy::Sequential,
            snarker_exe_path: "".into(),
            record: "none".into(),
            additional_ledgers_path: None,
        };
        let handle = std::thread::spawn(move || command.run().map_err(|e| e.to_string()));
        RustNode {
            _handle: handle,
            address,
            http_port,
            peer_id,
        }
    }

    fn state(&self) -> anyhow::Result<node::State> {
        let client = reqwest::blocking::Client::new();
        let mut address = self.address.clone();
        address.set_port(self.http_port);
        let url = format!("http://{address}/state");
        let response = client.get(url).send()?.json()?;
        Ok(response)
    }
}

impl Drop for RustNode {
    fn drop(&mut self) {}
}

impl MinaNode for RustNode {
    fn has_peer(&self, peerid: &str) -> anyhow::Result<bool> {
        let state = self.state()?;
        for (_, peer_data) in state.p2p.peers {
            if let P2pPeerStatus::Ready(_) = peer_data.status {
                if let Some(P2pConnectionOutgoingInitOpts::LibP2P(opts)) = peer_data.dial_opts {
                    eprintln!(
                        ">>> {} has peer {}",
                        self.address,
                        opts.peer_id.to_libp2p_string()
                    );
                    if opts.peer_id.to_libp2p_string() == peerid {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    fn address(&self) -> &SocketAddr {
        &self.address
    }

    fn peer_id(&self) -> &str {
        &self.peer_id
    }
}

trait MinaNode {
    fn peer_id(&self) -> &str;
    fn address(&self) -> &SocketAddr;
    fn has_peer(&self, peer_id: &str) -> anyhow::Result<bool>;
}

fn assert_p2p_ready(node: &impl MinaNode, duration: Duration) {
    let finish = Instant::now() + duration;
    while Instant::now() < finish {
        match TcpStream::connect(&node.address()) {
            Ok(_) => return,
            Err(e) => {
                eprintln!(">>> {}: not ready: {e}", node.address());
                std::thread::sleep(Duration::from_secs(10));
            }
        }
    }
    panic!("{}: p2p not ready", node.address());
}

fn assert_peers(node1: &impl MinaNode, node2: &impl MinaNode, duration: Duration) {
    let node1_peer_id = node1.peer_id();
    let node2_peer_id = node2.peer_id();
    let finish = Instant::now() + duration;
    while Instant::now() < finish {
        match node1.has_peer(node2_peer_id) {
            Ok(true) => {
                eprintln!(">>> {}: has peer {node2_peer_id}", node1.address());
            }
            Ok(false) => {
                eprintln!(">>> {}: no peer {node2_peer_id}", node1.address());
            }
            Err(e) => {
                eprintln!(">>> {}: peers not ready: {e}", node1.address());
            }
        }
        match node2.has_peer(node1_peer_id) {
            Ok(true) => {
                eprintln!(">>> {}: has peer {node1_peer_id}", node2.address());
                return;
            }
            Ok(false) => {
                eprintln!(">>> {}: no peer {node1_peer_id}", node2.address());
            }
            Err(e) => {
                eprintln!(">>> {}: peers not ready: {e}", node2.address());
            }
        }
        std::thread::sleep(Duration::from_secs(10));
    }
    panic!("{} and {} are not peers", node1.address(), node2.address());
}


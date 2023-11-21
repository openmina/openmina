use std::{
    ffi::OsStr,
    net::{SocketAddr, TcpStream},
    path::Path,
    process::{Child, Command},
    thread::JoinHandle,
    time::Duration,
};

use ledger::Instant;
use node::p2p::{connection::outgoing::P2pConnectionOutgoingInitOpts, P2pPeerStatus};

#[test]
#[ignore = "should be run manually"]
fn dicovery() {
    let d = temp_dir::TempDir::new().unwrap();
    let seed = run_seed(&d.path().join("seed"));
    assert_p2p_ready(&seed, Duration::from_secs(5 * 60));

    let rust = run_rust_node();
    assert_p2p_ready(&rust, Duration::from_secs(1 * 60));

    assert_has_peer(
        &rust,
        &SEED_ADDRESS.parse().unwrap(),
        SEED_PEERID.trim(),
        Duration::from_secs(20),
    );

    assert_has_peer(
        &seed,
        &RUST_ADDRESS.parse().unwrap(),
        RUST_PEERID.trim(),
        Duration::from_secs(20),
    );

    let ocaml = run_ocaml_node(&d.path().join("ocaml"));
    assert_p2p_ready(&ocaml, Duration::from_secs(5 * 60));

    assert_has_peer(
        &ocaml,
        &RUST_ADDRESS.parse().unwrap(),
        RUST_PEERID,
        Duration::from_secs(60 * 5),
    );
}

const SEED_ADDRESS: &str = "127.0.0.1:8302";
const OCAML_ADDRESS: &str = "127.0.0.1:18302";
const RUST_ADDRESS: &str = "127.0.0.1:28302";

const SEED_KEYPAIR_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/discovery/seed");
const OCAML_KEYPAIR_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/discovery/peer");

const SEED_PEERID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/discovery/seed.peerid"
));

const RUST_SECRET_KEY: &str = "5JJStLet4wfwdeWeK3UWHPbrQpF1r9xWk2oUPzjV3d1rjieTbGT";
const RUST_PEERID: &str = "12D3KooWFSGhnEp4itEq5YypEc868LxjyjubHJsEHdFLJMBncqhU";

fn run_seed(dir: &Path) -> OCamlNode {
    OCamlNode::run::<_, String>(
        SEED_ADDRESS.parse().unwrap(),
        SEED_KEYPAIR_FILE,
        dir,
        true,
        [],
    )
}

fn seed_multiaddr() -> String {
    let socket_addr = SEED_ADDRESS.parse::<SocketAddr>().unwrap();
    format!(
        "/ip4/{}/tcp/{}/p2p/{}",
        socket_addr.ip(),
        socket_addr.port(),
        SEED_PEERID.trim()
    )
}

fn run_ocaml_node(dir: &Path) -> OCamlNode {
    OCamlNode::run(
        OCAML_ADDRESS.parse().unwrap(),
        OCAML_KEYPAIR_FILE,
        dir,
        false,
        [seed_multiaddr()],
    )
}

fn run_rust_node() -> RustNode {
    RustNode::run(RUST_ADDRESS.parse().unwrap(), vec![seed_multiaddr()])
}

struct OCamlNode {
    process: Child,
    address: SocketAddr,
    graphql_port: u16,
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
    fn run<I: IntoIterator<Item = S>, S: AsRef<str>>(
        address: SocketAddr,
        libp2p_key: &str,
        dir: &Path,
        is_seed: bool,
        peers: I,
    ) -> OCamlNode {
        let mut command = Command::new("mina");
        let client_port = address.port() - 1;
        let graphql_port = address.port() + 1;
        command
            .arg("daemon")
            .args(["--libp2p-keypair", &libp2p_key])
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
        let child = command.spawn().unwrap();
        OCamlNode {
            process: child,
            address,
            graphql_port,
        }
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

    fn has_peer_json(address: &SocketAddr, peerid: &str, data: serde_json::Value) -> Option<bool> {
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

    fn has_peer(&self, address: &SocketAddr, peerid: &str) -> anyhow::Result<bool> {
        let json = self.grapql_query(PEERS_QUERY)?;
        let has_peer = Self::has_peer_json(address, peerid, json).expect("invalid json");
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
    handle: JoinHandle<Result<(), String>>,
    address: SocketAddr,
    http_port: u16,
}

impl RustNode {
    fn run<I: IntoIterator<Item = S>, S: AsRef<str>>(address: SocketAddr, peers: I) -> RustNode {
        let http_port = address.port() + 1;
        let command = cli::commands::node::Node {
            work_dir: "~/.openmina".into(),
            p2p_secret_key: Some(RUST_SECRET_KEY.parse().unwrap()),
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
            handle,
            address,
            http_port,
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
    fn has_peer(&self, address: &SocketAddr, peerid: &str) -> anyhow::Result<bool> {
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
}

trait MinaNode {
    fn address(&self) -> &SocketAddr;
    fn has_peer(&self, address: &SocketAddr, peer_id: &str) -> anyhow::Result<bool>;
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

fn assert_has_peer(node: &impl MinaNode, address: &SocketAddr, peer_id: &str, duration: Duration) {
    let finish = Instant::now() + duration;
    while Instant::now() < finish {
        match node.has_peer(address, peer_id) {
            Ok(true) => return,
            Ok(false) => {
                eprintln!(">>> {}: no peer {address}", node.address());
                std::thread::sleep(Duration::from_secs(10));
            }
            Err(e) => {
                eprintln!(">>> {}: peers not ready: {e}", node.address());
                std::thread::sleep(Duration::from_secs(10));
            }
        }
    }
    panic!("{}: no peer {address}", node.address());
}

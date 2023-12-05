use std::{
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command},
    time::Duration,
};

pub struct Node {
    child: Child,
    pub port: u16,
    pub peer_id: libp2p::PeerId,
    temp_dir: Option<temp_dir::TempDir>,
}

impl Node {
    pub fn local_addr(&self) -> libp2p::Multiaddr {
        format!("/ip4/127.0.0.1/tcp/{}/p2p/{}", self.port, self.peer_id)
            .parse()
            .expect("must be valid")
    }

    pub fn peer_id(&self) -> libp2p::PeerId {
        self.peer_id.clone()
    }

    pub fn kill(&mut self) {
        self.child.kill().expect("kill");
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

    pub fn generate_libp2p_keypair(dir: &Path) -> anyhow::Result<String> {
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

    pub fn spawn_with_temp_dir<I: IntoIterator<Item = S>, S: AsRef<str>>(
        p2p_port: u16,
        graphql_port: u16,
        client_port: u16,
        peers: I,
    ) -> anyhow::Result<Self> {
        let temp_dir = temp_dir::TempDir::new()?;
        let mut node = Node::spawn(p2p_port, client_port, graphql_port, &temp_dir.path(), peers)?;
        node.temp_dir = Some(temp_dir);
        Ok(node)
    }

    pub fn spawn<I: IntoIterator<Item = S>, S: AsRef<str>>(
        p2p_port: u16,
        graphql_port: u16,
        client_port: u16,
        dir: &Path,
        peers: I,
    ) -> anyhow::Result<Self> {
        let peer_id = match Self::read_peer_id(dir) {
            Ok(v) => v,
            Err(_) => Node::generate_libp2p_keypair(dir)?,
        };
        let peer_id = peer_id.parse()?;
        let mut cmd = Command::new("mina");

        cmd.env("MINA_LIBP2P_PASS", "");
        cmd.env("BPF_ALIAS", "auto-0.0.0.0");

        cmd.arg("daemon");
        cmd.arg("--config-dir").arg(&dir.join(".config"));
        cmd.arg("--config-file").arg("/var/lib/coda/berkeley.json");
        cmd.arg("--libp2p-keypair").arg(&Self::privkey_path(dir));
        cmd.args(["--external-ip", "127.0.0.1"])
            .args(["--external-port", &p2p_port.to_string()])
            .args(["--client-port", &client_port.to_string()])
            .args(["--rest-port", &graphql_port.to_string()]);
        let mut is_seed = true;
        for peer in peers {
            is_seed = false;
            cmd.args(["--peer", peer.as_ref()]);
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

        let prefix = format!("[localhost:{p2p_port}] ");
        let prefix2 = prefix.clone();
        std::thread::spawn(move || {
            if let Err(_) = Self::read_stream(stdout, std::io::stdout(), &prefix) {}
        });
        std::thread::spawn(move || {
            if let Err(_) = Self::read_stream(stderr, std::io::stderr(), &prefix2) {}
        });

        Ok(Self {
            child,
            port: p2p_port,
            peer_id,
            temp_dir: None,
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

    pub fn graphql_addr(&self) -> String {
        format!("http://127.0.0.1:{}/graphql", self.port + 1)
    }

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

    pub fn wait_for_p2p(&self, duration: Duration) -> anyhow::Result<bool> {
        let timeout = std::time::Instant::now() + duration;
        while std::time::Instant::now() < timeout {
            match TcpStream::connect(("127.0.0.1", self.port)) {
                Ok(_) => return Ok(true),
                Err(_) => {}
            }
            std::thread::sleep(Duration::from_secs(10));
        }
        return Ok(false);
    }
}

pub async fn wait_for_port_ready(port: u16, duration: Duration) -> anyhow::Result<bool> {
    let timeout = tokio::time::sleep(duration);
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let probe = tokio::task::spawn(async move {
        loop {
            interval.tick().await;
            match tokio::net::TcpStream::connect(("127.0.0.1", port)).await {
                Ok(_) => return,
                Err(_) => {}
            }
        }
    });
    let res = tokio::select! {
        _ = timeout => false,
        _ = probe => true,
    };
    Ok(res)
}

impl Drop for Node {
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

    let mut node =
        Node::spawn_with_temp_dir::<_, &str>(8302, 3086, 8301, []).expect("node is spawned");
    let stdout = node.child.stdout.take().unwrap();
    std::thread::spawn(move || {
        for line in BufRead::lines(BufReader::new(stdout)) {
            println!("{}", line.unwrap());
        }
    });

    node.child.wait().unwrap();
    node.kill();
}

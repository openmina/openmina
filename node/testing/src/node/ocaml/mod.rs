mod config;
pub use config::*;
use mina_p2p_messages::v2::StateHash;
use node::p2p::{
    connection::outgoing::{P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts},
    PeerId,
};

use std::{
    path::{Path, PathBuf},
    process::Child,
    time::Duration,
};

use serde::{Deserialize, Serialize};

pub struct OcamlNode {
    child: Child,
    executable: OcamlNodeExecutable,
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
    WaitReady { timeout: Duration },
    /// Kill ocaml node, cleaning up docker container if docker is used,
    /// without removing the work dir.
    Kill,
    /// Kill ocaml node, cleaning up docker container if docker is used.
    /// Along with it removing the work dir.
    KillAndRemove,
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
            DaemonJson::InMem(json) => {
                std::fs::write(&daemon_json_path, json.to_string()).map_err(|err| {
                    anyhow::anyhow!(
                        "failed to write InMem daemon.json to {}; error: {}",
                        daemon_json_path.display(),
                        err
                    )
                })?;
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
            executable: config.executable,
            libp2p_port: config.libp2p_port,
            graphql_port: config.graphql_port,
            peer_id,
            temp_dir: config.dir,
        })
    }

    pub fn dial_addr(&self) -> P2pConnectionOutgoingInitOpts {
        P2pConnectionOutgoingInitOpts::LibP2P(P2pConnectionOutgoingInitLibp2pOpts {
            peer_id: self.peer_id(),
            host: [127, 0, 0, 1].into(),
            port: self.libp2p_port,
        })
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id.into()
    }

    pub async fn exec(&mut self, step: OcamlStep) -> anyhow::Result<bool> {
        Ok(match step {
            OcamlStep::WaitReady { timeout } => {
                let t = redux::Instant::now();
                self.wait_for_p2p(timeout).await?;
                self.wait_for_synced(timeout - t.elapsed()).await?;
                true
            }
            OcamlStep::Kill | OcamlStep::KillAndRemove => {
                self.kill()?;
                true
            }
        })
    }

    fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }

    const PRIVKEY_PATH: &'static str = ".libp2p/key";
    const LIBP2P_KEYS: [(&'static str, &'static str); 10] = [
        (
            "12D3KooWKG1ZakBYEirEWdFYSstTEvxzTxCyuyaebojKthiESfWi",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"7fZJwAwzGgwFGipwjVBTiinHt5NfjTFjSnhk8rA","pwsalt":"9Gz996pkoUjJ8docbT7fJvJg16iL","pwdiff":[134217728,6],"ciphertext":"7oHFyFJd9kWTfH6R7GvbHD4WXBw5JQeiQYBGUpFhiPuiAyEgi1BVbczrzS6njWJ9FkRdNgZqwmSo23GPX4Zs27m3U66dJvyahendHCndG3Wu9wi8yaees78AQpbsU7JRa7U5DyCs9d34QLwpsgrGC2CqtDHJD3K3YncxVDjk4CCKeHseukZXUvkFToqY9CZRLHgYXR29hiB8JyTgoQ4maDDBdqBpFRb6Rjfb3LX8WEat6NpTjWi4A9uvNyDqk68a2aAo8ofjP811SBYxjZY3PMdD4hs5UAAZqbUNA"}"#,
        ),
        (
            "12D3KooWQoSDqpenkrjCnKCCLnwEMRVRqUfZJXqD8Y8awEnBAJZX",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"8psKzkQReBbhoWYriYSNGFVKsb7GMARxfSZLWDF","pwsalt":"9MdeSxJxE33e5sxYhnmPQNTDxbKj","pwdiff":[134217728,6],"ciphertext":"6YnXZjwJue344VEkLnFJ62VY9E2QxZPEtDoZSBXnNzJUEEK5MVjcC63GeM37kVXTVvoj8r9C9i4mUbTiwjpL9wg4NqxkcJpTMVBe6WDsYYrtt7S9o6p4xWAjm1hvbxXcsTzPN361amo2ZNCAuMGpCWQPnxeZ69bwLQkn4vKeGkdiUMdnziNfhKRaFcya4C1dNNoz8kAWFRexzZrjvSBymzZCvZPgof1mApyzcoWuYtAdENqbNURg2DBv53nLetmqA9zLTcDbXYE5hTgkVzMHa6qiia4xAhDbrax4r"}"#,
        ),
        (
            "12D3KooWEHSJwkn5ZdYVAcULpb3D3k6U2K4sU2BFh5xhoig1J4NK",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"84F8XwEf5k2yPqgC8NSTdf5wQEw7kmRgEdyss91","pwsalt":"BeSN6EXqWBCx9fndvnib5BYnNNdc","pwdiff":[134217728,6],"ciphertext":"7ECwoC7vK5QRKrPZvrBmaWrTKJiHKq9fCZr8YyVPgfHFS2VQ1BeKgQPvoc5JHNhy2Yju8PDDKzS3zHCdRSGaVRX49VrxjSU9wg4Fj7EWPGEs6VNCQahudCovGFd69iqS8WvcDSSw6acaEstSssTFYV3mUviDbkRA5HM9fUvE8SBkg9rHeeghCQ4qLK7cFRywCx9P6nronv5b5yy15xJDjAp6h7fwNxA7daXGa3E8dhtsE6FaPCtefkLBKJFXzuo3CMdRcBMZTt2XVGHS27rxtn1j5jeT9y34EMA5t"}"#,
        ),
        (
            "12D3KooWA1qcHYLWKZ7EUBMh3KKWbwys1DhH35WmH4y96scpntmv",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"6ifkqZVQ8MXHP5StG5umfJ2HhuSEpwxx33bRD4e","pwsalt":"9qREDkkiXgKV7VtryRcAkrzGY41p","pwdiff":[134217728,6],"ciphertext":"6uYbsTw56Vzaj4Jb9vWVQLTNHk92Qv6kFgZrqit8gUBya1M76U4wuNqQ1XMk5iKFSXFxf9hjtKN1NAbruUTnoySaCdCuLfjVJwUpDJFWwjRV3vmFjmZch9YjAL4H81z89V52BZkGpSantUxqGMLSrJjz8z5Rrr3C7ZCvooZddFieFkFfDtLBfhAsB3U2usMh83L6VMPg5Hawn7krdznSVzagiS1ENDPR92LfsCvxVVGxSNvUbBbjFCdroDb7eYi8mshiC3nGQWDYRQu9kQkHC6xsgarTsumkXnYpd"}"#,
        ),
        (
            "12D3KooWC14WLzaCT6fkR5WzzrayXDhmBpox7yWkif7Pg6Sk4uz1",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"7rGH8gCM5UL9LvatyUP6Lka8b4Ev2o5Mbe5ou7L","pwsalt":"BKcQuXkYx6H4xfcLLGM8ykJP5a3D","pwdiff":[134217728,6],"ciphertext":"7tZEvuCoXyXRVxh9NNrMCRLXgXj8MHxNgwoesztLQTagMzKhf48EepzUReYEViNC2EpWb2h7yoJdXMUbDGUuSQdoM1eF3qum9rHtmU4xdv8SGBEP9q9YHb1n8YS2SEr7WNcN3DsX7cqrzfnSjDsXNZaGzR5CbrK7g3NGv8RVyT1uZF2VHHeapDY6nFCyKN9nUJKpizbbRguR25QwWyx2nzcKF3mGq2iuCNVN5Z6gqzk7fhD8XFayxj57MqwvWTRD6pLRBJcmqCF5L9ZqpKEHAjXMcv3nwkKBHJaAF"}"#,
        ),
        (
            "12D3KooWRJEo19dU5eWgab1YrBPnK9HQA4SqDeQwx9NrankTcfSi",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"6VMHa83xqvbDzAppmfe9B8wn7dzzdAC1fyP5bXs","pwsalt":"9cXcTqJGi651tpA1ZBKSPVBP2h3q","pwdiff":[134217728,6],"ciphertext":"783EDXabmg2PmWhrSqcDog82NhWNMWasKC4o2d1oxVDTDxhmH5yGcjY74wV8HH16DpJw4ZxzW8gUCyC7Mhx1hEG8kc7wn38yBsoAqGfkA34g9n4FYJzwHvAB7on7zK3cveh2jXF3TTt3Etg4advpM7LvbY2eE9pz95TU1pCagu7haB83JHn2qSnfSCcMUTjS9copfJgkVD6YQYUxJmVi9erYUufjJqiF9p4ciCSuicU5SaJVB3rpSaRt1VgWMXeg47qWXVg98byNadHi8PgQZNnFifJc4FUDPtHDj"}"#,
        ),
        (
            "12D3KooWPTtAt3LXFqs8vbGL9VACn5wgcBm1jvaHZTGHnHzFq7c4",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"8q2jNrxkiwzvk8LWuU8zWy93APjitkaZUKr3fDP","pwsalt":"B5YVY1p2yFfGRsjs2us7WnmhWqPq","pwdiff":[134217728,6],"ciphertext":"7xxCXkAf3DokjZQ6CtwbuuXMBeYB5p6KxQAqaFLx96yBAsqQaK3EprR3xDKVR78x7Zrzj4NXWrFow2cg4xtze12SS43t46E3QhSsYPohcuZzKJe4agGJMDZVHaqd1aAPtJd2CX1fZCrWmxpa3hB72H2EKYPFSG1FYv77cYxU45aJx3V1XQAEQtoYKP9FmL95xogJHVHQSe2xWrvga8CLY4qtshJWkwHP1mV59xam9WhhZZjZkSThYTJMW9f4NTQ2EwRuud9zReLkh8fGEvfoxFjMsw8NCVxrtdTi5"}"#,
        ),
        (
            "12D3KooWLrjE3v7wZSCT4HsnXYmRsQnCWsaGcmU9mSJfPDRyjvpd",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"7zKo7kmPLjoJdvgxyMp2jguDnGDnyZ49TWfajiZ","pwsalt":"ARyFY4GSnyTC8ZjhvEaCSq3QeGsU","pwdiff":[134217728,6],"ciphertext":"7FfzcAPsEM7Lv5JzL7rYqgrHGV5bBDVVTEukULpDaCMGbHJDRFkgjxx6c6gbxCRbMJKma9yMrr5zxHsT9tfYCU7PAzqgVqa87TfphNXmqdSrNKWZVTS5SGMXAqku6vfJ19PA7TzJdr7oHZSjoim5Lh8r2x9iUTto7tdCBy4xWJQXE7aQYQ2ybB95DjzA3CtK7ypjxnZJDnvYq8zgXx9netbAX3NdTtsRDKQwNmjBzoQiKWd5jrqMigfFNcRTnJdpEn8jYMfa4fmXsnUe9ziXvjYdH9DJQA1354UQq"}"#,
        ),
        (
            "12D3KooWKPaZfwU42A9SDxyuDGWsLYiedudJvDAHY81UMXLLEgTe",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"7QAvnHnfp1MMJgEYN8VVwLuwWF3GEMUG5DA2KWP","pwsalt":"9YP8yrJRfF6w4efUSTZrPRhoV8xZ","pwdiff":[134217728,6],"ciphertext":"6i8Efh1z9AZi2ExYLFBLsPGSAXQT72pgQNP9yPBfDwqr8dtQxvqdh6V8qXwjyVSiWLGs6zbUFQj1mJ17F9irLzUTFLQ9WG98HveuUJLxv2WPoEekb2AUntAYNkbgVEmtWEYytWqYZnJUx5g1cnLo5ENzrcDTDcbHbQnYVkyH2GNpNrH4WqjK3TyDS2PzKwTwBFVMyhK4BUDVtjJWxfHnQNSaxT7gsGFNQTJjizxpqaqFk7Nc11GPaJmJFJKoVYD4ozCkeR2RKM9Dk7ct49vTQvULqqETwJmK1yMTf"}"#,
        ),
        (
            "12D3KooWKrWvpCzTJs45HS8c8Hbo9sfe65wBVCLWW2gyJDmnDDif",
            r#"{"box_primitive":"xsalsa20poly1305","pw_primitive":"argon2i","nonce":"8QEpCE7a48EdYXQ44ff9PXWDKw2AMmHMxc3nUaq","pwsalt":"8UUffeMbTFAwUwkamE1xWoVTJqDi","pwdiff":[134217728,6],"ciphertext":"93Wdv4Xq9794GWuZPZUPug2gfPpXD2dnLxSx9jGjtr7rTGv14W7JPieGJJ4zTw6T54x1NwyhH9HcDwsQxmUT364KibpbczuA9bnTFcp6ahoYpetrHB8FJTk7TprkmazqprJm7QDqJ97jyE7PuVNWg9NSbMRzet1c5Jxk2qfUYVdtSNgcQB5J5oUTibL6fc5UKZmfBoSixw3E3QFPnBRN8W7X3nfcHykK9eck2u5YJrv1gRYoupp2EX1CjWwKp3ebDa9bLLZiWTTSBKsj7uhLh5aCxgHpoPCyNcnaq"}"#,
        ),
    ];

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
        use std::{
            fs::OpenOptions,
            io::Write,
            os::unix::fs::{DirBuilderExt, OpenOptionsExt},
        };

        let (peer_id, key) = Self::LIBP2P_KEYS[config.libp2p_keypair_i];
        let privkey_path = Self::privkey_path(dir);
        let privkey_parent_dir = privkey_path.as_path().parent().unwrap();
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(privkey_parent_dir)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o600)
            .open(&privkey_path)?;
        file.write_all(key.as_bytes())?;
        std::fs::write(
            privkey_path.with_extension("peerid"),
            format!("peerid:{peer_id}"),
        )?;
        Ok(peer_id.to_owned())
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

    /// Queries graphql to get chain_id.
    pub fn chain_id(&self) -> anyhow::Result<String> {
        let res = self.grapql_query("query { daemonStatus { chainId } }")?;
        res["data"]["daemonStatus"]["chainId"]
            .as_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| anyhow::anyhow!("empty chain_id response"))
    }

    /// Queries graphql to get chain_id.
    pub async fn chain_id_async(&self) -> anyhow::Result<String> {
        let res = self.grapql_query_async("query { daemonStatus { chainId } }").await?;
        res["data"]["daemonStatus"]["chainId"]
            .as_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| anyhow::anyhow!("empty chain_id response"))
    }

    /// Queries graphql to check if ocaml node is synced,
    /// returning it's best tip hash if yes.
    pub fn synced_best_tip(&self) -> anyhow::Result<Option<StateHash>> {
        let mut res = self.grapql_query("query { daemonStatus { syncStatus, stateHash } }")?;
        let data = &mut res["data"]["daemonStatus"];
        if data["syncStatus"].as_str() == Some("SYNCED") {
            Ok(Some(serde_json::from_value(data["stateHash"].take())?))
        } else {
            Ok(None)
        }
    }

    /// Queries graphql to check if ocaml node is synced,
    /// returning it's best tip hash if yes.
    pub async fn synced_best_tip_async(&self) -> anyhow::Result<Option<StateHash>> {
        let mut res = self.grapql_query_async("query { daemonStatus { syncStatus, stateHash } }").await?;
        let data = &mut res["data"]["daemonStatus"];
        if data["syncStatus"].as_str() == Some("SYNCED") {
            Ok(Some(serde_json::from_value(data["stateHash"].take())?))
        } else {
            Ok(None)
        }
    }

    fn graphql_addr(&self) -> String {
        format!("http://127.0.0.1:{}/graphql", self.graphql_port)
    }

    // TODO(binier): shouldn't be publically accessible.
    //
    // Only `exec` function should be exposed and instead of this, we
    // should have a step to query graphql and assert response as a part
    // of that step.
    pub async fn grapql_query_async(&self, query: &str) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let response = client
            .post(self.graphql_addr())
            .json(&{
                serde_json::json!({
                    "query": query
                })
            })
            .send().await?;

        Ok(response.json().await?)
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
            _ = timeout_fut => anyhow::bail!("waiting for ocaml node's p2p port to be ready timed out! timeout: {timeout:?}"),
            _ = probe => Ok(()),
        }
    }

    async fn wait_for_synced(&self, timeout: Duration) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        tokio::time::timeout(timeout, async {
            loop {
                interval.tick().await;
                if self.synced_best_tip_async().await.map_or(false, |tip| tip.is_some()) {
                    return;
                }
            }
        })
        .await
        .map_err(|_| {
            anyhow::anyhow!("waiting for ocaml node to be synced timed out! timeout: {timeout:?}")
        })
    }
}

impl Drop for OcamlNode {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Err(err) => {
                eprintln!("error getting status from OCaml node: {err}");
            }
            Ok(None) => {
                self.executable.kill(&self.temp_dir);
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
        libp2p_keypair_i: 0,
        libp2p_port: 8302,
        graphql_port: 3086,
        client_port: 8301,
        initial_peers: Vec::new(),
        daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
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

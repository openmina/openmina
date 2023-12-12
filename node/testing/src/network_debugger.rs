use std::{
    collections::VecDeque,
    net::SocketAddr,
    process::{Child, Command},
    time::SystemTime,
};

use reqwest::blocking::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};

pub struct Debugger {
    child: Option<Child>,
    host: &'static str,
    port: u16,
    client: Client,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Connection {
    pub info: ConnectionInfo,
    pub incoming: bool,
    pub timestamp: SystemTime,
    pub timestamp_close: SystemTime,

    pub alias: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ConnectionInfo {
    pub addr: SocketAddr,
    pub pid: u32,
    pub fd: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FullMessage {
    pub connection_id: u64,
    pub remote_addr: String,
    pub incoming: bool,
    pub timestamp: SystemTime,
    pub stream_id: StreamId,
    pub stream_kind: String,
    pub message: serde_json::Value,
    pub size: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum StreamId {
    Handshake,
    Forward(u64),
    Backward(u64),
}

impl Debugger {
    pub fn drone_ci() -> Self {
        Debugger {
            child: None,
            host: "localhost",
            port: 8000,
            client: ClientBuilder::new().build().unwrap(),
        }
    }

    pub fn spawn(port: u16) -> Self {
        let mut cmd = Command::new("bpf-recorder");
        cmd.env("SERVER_PORT", port.to_string());
        Debugger {
            child: Some(cmd.spawn().expect("cannot spawn debugger")),
            host: "localhost",
            port,
            client: ClientBuilder::new().build().unwrap(),
        }
    }

    pub fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            use nix::{
                sys::signal::{self, Signal},
                unistd::Pid,
            };

            if let Err(err) = signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT) {
                eprintln!("error sending ctrl+c to Network debugger: {err}");
            }
            match child.try_wait() {
                Err(err) => {
                    eprintln!("error getting status from Network debugger: {err}");
                }
                Ok(None) => {
                    eprintln!("error getting status from Network debugger");
                }
                Ok(Some(status)) => {
                    eprintln!("network debugger {status}");
                }
            }
        }
    }

    pub fn get_message(&self, id: u64) -> anyhow::Result<Vec<u8>> {
        let port = self.port;
        let host = self.host;
        self.client
            .get(&format!("http://{host}:{port}/message_bin/{id}"))
            .send()?
            .bytes()
            .map(|x| x.to_vec())
            .map_err(Into::into)
    }

    pub fn get_connections(&self, params: &str) -> anyhow::Result<Vec<(u64, Connection)>> {
        let port = self.port;
        let host = self.host;
        let res = self
            .client
            .get(&format!("http://{host}:{port}/connections?{params}"))
            .send()?
            .text()?;
        serde_json::from_str::<Vec<_>>(&res).map_err(From::from)
    }

    pub fn get_messages(&self, params: &str) -> anyhow::Result<Vec<(u64, FullMessage)>> {
        let port = self.port;
        let host = self.host;
        let res = self
            .client
            .get(&format!("http://{host}:{port}/messages?{params}"))
            .send()?
            .text()?;
        serde_json::from_str::<Vec<(u64, FullMessage)>>(&res).map_err(From::from)
    }

    pub fn current_cursor(&self) -> u64 {
        self.get_messages("direction=reverse&limit=1")
            .map_err(|err| eprintln!("determine cursor error: {err}"))
            .ok()
            .and_then(|msgs| msgs.first().map(|(id, _)| *id))
            .unwrap_or_default()
    }

    pub fn connections_raw(&self, cursor: u64) -> Connections<'_> {
        Connections {
            inner: self,
            cursor,
            buffer: VecDeque::default(),
        }
    }

    pub fn connections(&self) -> ConnectionsHandshaked<'_> {
        ConnectionsHandshaked {
            messages: self.messages(0, "stream_kind=/noise"),
        }
    }

    pub fn messages(&self, cursor: u64, additional: &'static str) -> Messages<'_> {
        Messages {
            inner: self,
            cursor,
            additional,
            buffer: VecDeque::default(),
        }
    }
}

pub struct ConnectionsHandshaked<'a> {
    messages: Messages<'a>,
}

impl<'a> Iterator for ConnectionsHandshaked<'a> {
    type Item = (u64, Connection);

    fn next(&mut self) -> Option<Self::Item> {
        let (_, msg) = self.messages.next()?;
        let id = msg.connection_id;

        let port = self.messages.inner.port;
        let host = self.messages.inner.host;
        let res = self
            .messages
            .inner
            .client
            .get(&format!("http://{host}:{port}/connection/{id}"))
            .send()
            .ok()?
            .text()
            .ok()?;
        serde_json::from_str::<Connection>(&res)
            .ok()
            .map(|cn| (id, cn))
    }
}

pub struct Connections<'a> {
    inner: &'a Debugger,
    cursor: u64,
    buffer: VecDeque<(u64, Connection)>,
}

impl<'a> Iterator for Connections<'a> {
    type Item = (u64, Connection);

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            let params = format!("direction=forward&limit=100&id={}", self.cursor);
            let msgs = self
                .inner
                .get_connections(&params)
                .map_err(|err| eprintln!("{err}"))
                .ok()?;
            let (last_id, _) = msgs.last()?;
            self.cursor = *last_id + 1;
            self.buffer.extend(msgs);
        }
        self.buffer.pop_front()
    }
}

pub struct Messages<'a> {
    inner: &'a Debugger,
    cursor: u64,
    additional: &'static str,
    buffer: VecDeque<(u64, FullMessage)>,
}

impl<'a> Iterator for Messages<'a> {
    type Item = (u64, FullMessage);

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            let params = if self.additional.is_empty() {
                format!("direction=forward&limit=100&id={}", self.cursor)
            } else {
                format!(
                    "direction=forward&limit=100&id={}&{}",
                    self.cursor, self.additional
                )
            };

            let msgs = self
                .inner
                .get_messages(&params)
                .map_err(|err| eprintln!("{err}"))
                .ok()?;
            let (last_id, _) = msgs.last()?;
            self.cursor = *last_id + 1;
            self.buffer.extend(msgs);
        }
        self.buffer.pop_front()
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        self.kill();
    }
}

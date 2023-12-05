use std::{
    collections::VecDeque,
    process::{Child, Command},
    time::SystemTime,
};

use reqwest::blocking::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};

pub struct Debugger {
    child: Child,
    port: u16,
    client: Client,
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
    pub fn spawn(port: u16) -> Self {
        let mut cmd = Command::new("bpf-recorder");
        cmd.env("SERVER_PORT", port.to_string())
            .env("DB_PATH", "/tmp/db");
        Debugger {
            child: cmd.spawn().expect("cannot spawn debugger"),
            port,
            client: ClientBuilder::new().build().unwrap(),
        }
    }

    pub fn kill(&mut self) {
        if let Err(err) = self.child.kill() {
            eprintln!("error send signal to the debugger: {err}");
        }
    }

    pub fn get_message(&self, id: u64) -> anyhow::Result<Vec<u8>> {
        let port = self.port;
        self.client
            .get(&format!("http://localhost:{port}/message_bin/{id}"))
            .send()?
            .bytes()
            .map(|x| x.to_vec())
            .map_err(Into::into)
    }

    pub fn get_messages(&self, params: &str) -> anyhow::Result<Vec<(u64, FullMessage)>> {
        let port = self.port;

        let res = self
            .client
            .get(&format!("http://localhost:{port}/messages?{params}"))
            .send()?
            .text()?;
        serde_json::from_str::<Vec<(u64, FullMessage)>>(&res).map_err(From::from)
    }

    pub fn messages(&self) -> Messages<'_> {
        Messages {
            inner: self,
            cursor: 0,
            buffer: VecDeque::default(),
        }
    }
}

pub struct Messages<'a> {
    inner: &'a Debugger,
    cursor: u64,
    buffer: VecDeque<(u64, FullMessage)>,
}

impl<'a> Iterator for Messages<'a> {
    type Item = (u64, FullMessage);

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            let params = format!("limit=100&cursor={}", self.cursor);
            let msgs = self.inner.get_messages(&params).ok()?;
            let (last_id, _) = msgs.last()?;
            self.cursor = *last_id + 1;
            self.buffer.extend(msgs);
        }
        self.buffer.pop_back()
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Err(err) => {
                eprintln!("error getting status from Network debugger: {err}");
            }
            Ok(None) => {
                if let Err(err) = self.child.kill() {
                    eprintln!("error killing Network debugger: {err}");
                } else if let Err(err) = self.child.wait() {
                    eprintln!("error getting status from Network debugger: {err}");
                }
            }
            _ => {}
        }
    }
}

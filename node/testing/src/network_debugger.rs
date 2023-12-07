use std::{
    collections::VecDeque,
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
            host: "debugger",
            port: 8000,
            client: ClientBuilder::new().build().unwrap(),
        }
    }

    pub fn spawn(port: u16) -> Self {
        let mut cmd = Command::new("bpf-recorder");
        cmd.env("SERVER_PORT", port.to_string())
            .env("DB_PATH", "/tmp/db");
        Debugger {
            child: Some(cmd.spawn().expect("cannot spawn debugger")),
            host: "localhost",
            port,
            client: ClientBuilder::new().build().unwrap(),
        }
    }

    pub fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            if let Err(err) = child.kill() {
                eprintln!("error send signal to the debugger: {err}");
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
            .ok()
            .and_then(|msgs| msgs.first().map(|(id, _)| *id))
            .unwrap_or_default()
    }

    pub fn messages(&self, cursor: u64) -> Messages<'_> {
        Messages {
            inner: self,
            cursor,
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
            let params = format!("limit=100&id={}", self.cursor);
            // TODO: log error?
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
        if let Some(mut child) = self.child.take() {
            match child.try_wait() {
                Err(err) => {
                    eprintln!("error getting status from Network debugger: {err}");
                }
                Ok(None) => {
                    if let Err(err) = child.kill() {
                        eprintln!("error killing Network debugger: {err}");
                    } else if let Err(err) = child.wait() {
                        eprintln!("error getting status from Network debugger: {err}");
                    }
                }
                _ => {}
            }
        }
    }
}

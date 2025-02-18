pub use tokio::sync::{mpsc, oneshot};

pub mod broadcast {
    pub use tokio::sync::broadcast::*;

    #[deprecated(note = "don't use across threads as it can cause panic in WASM")]
    #[inline(always)]
    pub fn channel<T: Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        tokio::sync::broadcast::channel(capacity)
    }
}

pub mod watch {
    pub use tokio::sync::watch::*;

    #[deprecated(note = "don't use across threads as it can cause panic in WASM")]
    #[inline(always)]
    pub fn channel<T>(init: T) -> (Sender<T>, Receiver<T>) {
        tokio::sync::watch::channel(init)
    }
}

#[allow(dead_code)]
pub struct Aborter(mpsc::Receiver<()>, mpsc::Sender<()>);

#[derive(Clone)]
pub struct Aborted(mpsc::Sender<()>);

impl Default for Aborter {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(1);
        Self(rx, tx)
    }
}

impl Aborter {
    pub fn listener_count(&self) -> usize {
        self.1.strong_count().saturating_sub(1)
    }

    /// Simply drops the object. No need to call manually, unless you
    /// temporarily have to retain object for some reason.
    pub fn abort_mut(&mut self) {
        std::mem::take(self);
    }

    pub fn aborted(&self) -> Aborted {
        Aborted(self.1.clone())
    }
}

impl Aborted {
    pub async fn wait(&self) {
        self.0.closed().await;
    }
}

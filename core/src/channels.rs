pub use tokio::sync::oneshot;

pub mod mpsc {
    use std::sync::{Arc, Weak};

    pub use flume::{SendError, TryRecvError, TrySendError};

    pub type RecvStream<T> = flume::r#async::RecvStream<'static, T>;

    pub struct Sender<T>(flume::Sender<T>);
    pub struct Receiver<T>(flume::Receiver<T>);

    pub struct UnboundedSender<T>(flume::Sender<T>, Arc<()>);
    pub struct UnboundedReceiver<T>(flume::Receiver<T>);

    pub type TrackedUnboundedSender<T> = UnboundedSender<Tracked<T>>;
    pub type TrackedUnboundedReceiver<T> = UnboundedReceiver<Tracked<T>>;

    #[allow(dead_code)]
    pub struct Tracked<T>(pub T, pub Tracker);
    #[allow(dead_code)]
    pub struct Tracker(Weak<()>);

    impl<T> std::fmt::Debug for UnboundedSender<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?} (len: {})", self.0, self.len())
        }
    }

    impl<T> std::fmt::Debug for UnboundedReceiver<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?} (len: {})", self.0, self.len())
        }
    }

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Clone for UnboundedSender<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }

    impl<T> std::ops::Deref for Tracked<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> std::ops::DerefMut for Tracked<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<T> Sender<T> {
        pub async fn send(&self, message: T) -> Result<(), SendError<T>> {
            self.0.send_async(message).await
        }

        pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
            self.0.try_send(message)
        }
    }

    impl<T> Receiver<T> {
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        pub fn len(&self) -> usize {
            self.0.len()
        }

        pub async fn recv(&mut self) -> Option<T> {
            self.0.recv_async().await.ok()
        }

        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            self.0.try_recv()
        }
    }

    impl<T> UnboundedSender<T> {
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        pub fn len(&self) -> usize {
            Arc::weak_count(&self.1)
        }

        pub fn send(&self, message: T) -> Result<(), SendError<T>> {
            self.0.send(message)
        }
    }

    impl<T> TrackedUnboundedSender<T> {
        pub fn tracked_send(&self, message: T) -> Result<(), SendError<T>> {
            let msg = Tracked(message, Tracker(Arc::downgrade(&self.1)));
            self.send(msg).map_err(|err| SendError(err.0 .0))
        }
    }

    impl<T> UnboundedReceiver<T> {
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        pub fn len(&self) -> usize {
            self.0.len()
        }

        pub async fn recv(&mut self) -> Option<T> {
            self.0.recv_async().await.ok()
        }

        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            self.0.try_recv()
        }

        pub fn stream(&self) -> RecvStream<T> {
            self.0.clone().into_stream()
        }

        pub fn blocking_recv(&mut self) -> Option<T> {
            self.0.recv().ok()
        }
    }

    pub fn channel<T>(bound: usize) -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = flume::bounded(bound);

        (Sender(tx), Receiver(rx))
    }

    pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
        let (tx, rx) = flume::unbounded();

        (UnboundedSender(tx, Arc::new(())), UnboundedReceiver(rx))
    }

    pub fn tracked_unbounded_channel<T>(
    ) -> (UnboundedSender<Tracked<T>>, UnboundedReceiver<Tracked<T>>) {
        let (tx, rx) = flume::unbounded();

        (UnboundedSender(tx, Arc::new(())), UnboundedReceiver(rx))
    }
}

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
pub struct Aborter(flume::Receiver<()>, flume::Sender<()>);

#[derive(Clone)]
pub struct Aborted(flume::Sender<()>);

impl Default for Aborter {
    fn default() -> Self {
        let (tx, rx) = flume::bounded(0);
        Self(rx, tx)
    }
}

impl Aborter {
    pub fn listener_count(&self) -> usize {
        self.0.sender_count().saturating_sub(1)
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
        // it returning an error means receiver was dropped
        while self.0.send_async(()).await.is_ok() {}
    }
}

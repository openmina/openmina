use redux::Instant;

use ::libp2p::futures::channel::mpsc;
use ::libp2p::futures::stream::StreamExt;

use lib::event_source::Event;

pub mod libp2p;
use self::libp2p::Libp2pService;

pub mod rpc;
use self::rpc::RpcService;

pub struct EventReceiver {
    rx: mpsc::Receiver<Event>,
    queue: Vec<Event>,
}

impl EventReceiver {
    /// If `Err(())`, `mpsc::Sender` for this channel was dropped.
    pub async fn wait_for_events(&mut self) -> Result<(), ()> {
        let next = self.rx.next().await.ok_or(())?;
        self.queue.push(next);
        Ok(())
    }

    pub fn has_next(&mut self) -> bool {
        if self.queue.is_empty() {
            if let Some(event) = self.try_next() {
                self.queue.push(event);
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn try_next(&mut self) -> Option<Event> {
        if !self.queue.is_empty() {
            Some(self.queue.remove(0))
        } else {
            self.rx.try_next().ok().flatten()
        }
    }
}

impl From<mpsc::Receiver<Event>> for EventReceiver {
    fn from(rx: mpsc::Receiver<Event>) -> Self {
        Self {
            rx,
            queue: Vec::with_capacity(1),
        }
    }
}

pub struct NodeWasmService {
    pub event_source_sender: mpsc::Sender<Event>,
    pub event_source_receiver: EventReceiver,

    pub libp2p: Libp2pService,
    pub rpc: RpcService,
}

impl lib::Service for NodeWasmService {}
impl redux::Service for NodeWasmService {}
impl lib::service::TimeService for NodeWasmService {
    fn monotonic_time(&mut self) -> Instant {
        redux::Instant::now()
    }
}

impl lib::service::EventSourceService for NodeWasmService {
    fn next_event(&mut self) -> Option<Event> {
        self.event_source_receiver.try_next()
    }
}

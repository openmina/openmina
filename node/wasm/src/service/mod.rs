use std::sync::Arc;

use ::libp2p::futures::channel::{mpsc, oneshot};
use ::libp2p::futures::stream::StreamExt;
use ::libp2p::futures::SinkExt;
use redux::Instant;
use wasm_bindgen_futures::spawn_local;

use lib::event_source::{Event, SnarkEvent};
use lib::snark::block_verify::SnarkBlockVerifyError;

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

impl lib::service::SnarkBlockVerifyService for NodeWasmService {
    fn verify_init(
        &mut self,
        req_id: lib::snark::block_verify::SnarkBlockVerifyId,
        verifier_index: Arc<lib::snark::VerifierIndex>,
        verifier_srs: Arc<lib::snark::VerifierSRS>,
        block: &mina_p2p_messages::v2::MinaBlockHeaderStableV2,
    ) {
        let mut tx = self.event_source_sender.clone();
        // TODO(binier): pass block as `Arc` to avoid extra cloning.
        let block = Arc::new(block.clone());

        let (mut tx, rx) = oneshot::channel();
        rayon::spawn_fifo(move || {
            let result = {
                if !lib::snark::accumulator_check(&verifier_srs, &block.protocol_state_proof) {
                    Err(SnarkBlockVerifyError::AccumulatorCheckFailed)
                } else if !lib::snark::verify(&block, &verifier_index) {
                    Err(SnarkBlockVerifyError::VerificationFailed)
                } else {
                    Ok(())
                }
            };

            let _ = tx.send(result);
        });

        let mut tx = self.event_source_sender.clone();
        spawn_local(async move {
            let result = match rx.await {
                Ok(v) => v,
                Err(_) => Err(SnarkBlockVerifyError::ValidatorThreadCrashed),
            };
            tx.send(SnarkEvent::BlockVerify(req_id, result).into())
                .await;
        });
    }
}

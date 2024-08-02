use std::time::Duration;

use node::{Effects, EventSourceAction, Service, State, Store};

use crate::{
    rpc::{RpcReceiver, RpcSender},
    EventReceiver, NodeServiceCommon,
};

pub struct Node<Serv> {
    store: Store<Serv>,
}

impl<Serv: Service + AsMut<NodeServiceCommon>> Node<Serv> {
    pub fn new(
        rng_seed: [u8; 32],
        initial_state: State,
        mut service: Serv,
        override_effects: Option<Effects<Serv>>,
    ) -> Self {
        let p2p_sec_key = service.as_mut().p2p.sec_key.clone();
        service
            .recorder()
            .initial_state(rng_seed, p2p_sec_key, &initial_state);

        let time_since_epoch = initial_state
            .time()
            .checked_sub(redux::Timestamp::ZERO)
            .unwrap();
        let store = Store::new(
            node::reducer,
            override_effects.unwrap_or(node::effects),
            service,
            redux::SystemTime::UNIX_EPOCH + time_since_epoch,
            initial_state,
        );

        Self { store }
    }

    pub fn store(&self) -> &Store<Serv> {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut Store<Serv> {
        &mut self.store
    }

    pub fn state(&self) -> &State {
        self.store().state.get()
    }

    fn service_mut(&mut self) -> &mut Serv {
        &mut self.store.service
    }

    fn service_common_mut(&mut self) -> &mut NodeServiceCommon {
        self.service_mut().as_mut()
    }

    fn event_receiver_with_rpc_receiver(&mut self) -> (&mut EventReceiver, &mut RpcReceiver) {
        self.service_common_mut().event_receiver_with_rpc_receiver()
    }

    fn event_receiver(&mut self) -> &mut EventReceiver {
        &mut self.service_common_mut().event_receiver
    }

    pub async fn run_forever(&mut self) {
        loop {
            self.store_mut().dispatch(EventSourceAction::WaitForEvents);

            let (event_receiver, rpc_receiver) = self.event_receiver_with_rpc_receiver();
            let wait_for_events = event_receiver.wait_for_events();
            let rpc_req_fut = async {
                // TODO(binier): optimize maybe to not check it all the time.
                match rpc_receiver.recv().await {
                    Some(v) => v,
                    None => std::future::pending().await,
                }
            };
            let timeout = tokio::time::sleep(Duration::from_millis(100));

            tokio::select! {
                _ = wait_for_events => {
                    while self.event_receiver().has_next() {
                        self.store_mut().dispatch(EventSourceAction::ProcessEvents);
                    }
                }
                req = rpc_req_fut => {
                    self.service_common_mut().process_rpc_request(req);
                }
                _ = timeout => {
                    self.store_mut().dispatch(EventSourceAction::WaitTimeout);
                }
            }
        }
    }

    pub fn rpc(&mut self) -> RpcSender {
        self.service_common_mut().rpc_sender()
    }
}

impl<Serv> Clone for Node<Serv>
where
    Serv: Clone,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}

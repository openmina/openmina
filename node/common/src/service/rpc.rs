use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use node::rpc::{
    RpcBlockProducerStatsGetResponse, RpcDiscoveryBoostrapStatsResponse,
    RpcDiscoveryRoutingTableResponse, RpcHealthCheckResponse, RpcLedgerAccountsResponse,
    RpcMessageProgressResponse, RpcPeersGetResponse, RpcReadinessCheckResponse, RpcRequest,
    RpcStateGetError, RpcStatusGetResponse, RpcTransactionInjectFailure,
    RpcTransactionPoolResponse, RpcTransitionFrontierUserCommandsResponse,
};
use serde::{Deserialize, Serialize};

use node::core::channels::{mpsc, oneshot};
use node::core::requests::PendingRequests;
use node::p2p::connection::P2pConnectionResponse;
pub use node::rpc::{
    ActionStatsResponse, RespondError, RpcActionStatsGetResponse, RpcId, RpcIdType,
    RpcP2pConnectionOutgoingResponse, RpcScanStateSummaryGetResponse, RpcSnarkPoolGetResponse,
    RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse, RpcStateGetResponse,
    RpcSyncStatsGetResponse, RpcTransactionInjectResponse,
};
use node::State;
use node::{event_source::Event, rpc::RpcSnarkPoolJobGetResponse};

use crate::NodeService;

#[derive(Serialize, Deserialize, Debug)]
pub enum RpcP2pConnectionIncomingResponse {
    Answer(P2pConnectionResponse),
    Result(Result<(), String>),
}

pub struct NodeRpcRequest {
    pub req: RpcRequest,
    pub responder: Box<dyn Send + std::any::Any>,
}

#[derive(Clone)]
pub struct RpcSender {
    tx: mpsc::Sender<NodeRpcRequest>,
}

pub type RpcReceiver = mpsc::Receiver<NodeRpcRequest>;

pub struct RpcService {
    pending: PendingRequests<RpcIdType, Box<dyn Send + std::any::Any>>,

    req_sender: mpsc::Sender<NodeRpcRequest>,
    req_receiver: mpsc::Receiver<NodeRpcRequest>,
}

impl RpcSender {
    pub fn new(tx: mpsc::Sender<NodeRpcRequest>) -> Self {
        Self { tx }
    }

    pub async fn oneshot_request<T>(&self, req: RpcRequest) -> Option<T>
    where
        T: 'static + Send + Serialize,
    {
        let (tx, rx) = oneshot::channel::<T>();
        let responder = Box::new(tx);
        let sender = self.tx.clone();
        let _ = sender.send(NodeRpcRequest { req, responder }).await;

        rx.await.ok()
    }

    pub async fn multishot_request<T>(
        &self,
        expected_messages: usize,
        req: RpcRequest,
    ) -> mpsc::Receiver<T>
    where
        T: 'static + Send + Serialize,
    {
        let (tx, rx) = mpsc::channel::<T>(expected_messages);
        let responder = Box::new(tx);
        let sender = self.tx.clone();
        let _ = sender.send(NodeRpcRequest { req, responder }).await;

        rx
    }

    pub async fn peer_connect(
        &self,
        opts: P2pConnectionOutgoingInitOpts,
    ) -> Result<String, String> {
        let peer_id = opts.peer_id().to_string();
        let req = RpcRequest::P2pConnectionOutgoing(opts);
        self.oneshot_request::<RpcP2pConnectionOutgoingResponse>(req)
            .await
            .ok_or_else(|| "state machine shut down".to_owned())??;

        Ok(peer_id)
    }
}

impl Default for RpcService {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        Self {
            pending: Default::default(),
            req_sender: tx,
            req_receiver: rx,
        }
    }

    /// Channel for sending the rpc request to state machine.
    pub fn req_sender(&self) -> RpcSender {
        RpcSender::new(self.req_sender.clone())
    }

    /// Channel for receiving rpc requests in state machine.
    pub fn req_receiver(&mut self) -> &mut RpcReceiver {
        &mut self.req_receiver
    }

    pub fn process_request(&mut self, req: NodeRpcRequest) -> Event {
        let rpc_id = self.pending.add(req.responder);
        let req = req.req;
        Event::Rpc(rpc_id, Box::new(req))
    }
}

impl NodeService {
    pub fn process_rpc_request(&mut self, req: NodeRpcRequest) {
        let rpc_id = self.rpc.pending.add(req.responder);
        let req = req.req;
        let tx = self.event_sender.clone();

        let _ = tx.send(Event::Rpc(rpc_id, Box::new(req)));
    }
}

macro_rules! rpc_service_impl {
    ($name:ident, $ty:ty) => {
        fn $name(&mut self, rpc_id: RpcId, response: $ty) -> Result<(), RespondError> {
            let entry = self.rpc.pending.remove(rpc_id);
            let chan = entry.ok_or(RespondError::UnknownRpcId)?;
            let chan = chan
                .downcast::<oneshot::Sender<$ty>>()
                .or(Err(RespondError::UnexpectedResponseType))?;
            chan.send(response)
                .or(Err(RespondError::RespondingFailed))?;
            Ok(())
        }
    };
}

macro_rules! state_field_filter {
    ($state:expr, $($part:ident)|*, $filter:expr ) => {
        $(
            if let Some(filter) = strip_root_field($filter, stringify!($part)) {
                (serde_json::to_value(&$state.$part)?, format!("${filter}"))
            } else
        )*
        {
            (serde_json::to_value($state)?, $filter.to_string())
        }
    };
}

/// Strips topmost field name `field` from the jsonpath expression `filter`,
/// returning modified filter. If the `filter` does not start with the specified
/// field, returns [None].
///
/// ```ignore
/// use openmina_node_native::rpc::strip_root_field;
///
/// let filter = strip_root_field("$.field", "field");
/// assert_eq!(filter, Some(""));
///
/// let filter = strip_root_field("$.field.another", "field");
/// assert_eq!(filter, Some(".another"));
///
/// let filter = strip_root_field("$.field_other", "field");
/// assert_eq!(filter, None);
/// ```
fn strip_root_field<'a>(filter: &'a str, field: &str) -> Option<&'a str> {
    let strip_root = |f: &'a str| f.strip_prefix('$');
    let field_char = |c: char| c.is_alphabetic() || c == '_';
    let strip_dot_field = |f: &'a str| {
        f.strip_prefix('.').and_then(|f| {
            f.strip_prefix(field)
                .and_then(|f| (!f.starts_with(field_char)).then_some(f))
        })
    };
    let strip_index_field = |f: &'a str| {
        f.strip_prefix("['")
            .and_then(|f| f.strip_prefix(field))
            .and_then(|f| f.strip_prefix("']"))
    };
    strip_root(filter).and_then(|f| strip_dot_field(f).or_else(|| strip_index_field(f)))
}

fn optimize_filtered_state(
    state: &State,
    filter: &str,
) -> Result<(serde_json::Value, String), serde_json::Error> {
    let (value, filter) = state_field_filter!(
        state,
        config
            | p2p
            | snark
            | consensus
            | transition_frontier
            | snark_pool
            | external_snark_worker
            | block_producer
            | rpc
            | watched_accounts,
        filter
    );
    Ok((value, filter))
}

impl node::rpc::RpcService for NodeService {
    fn respond_state_get(
        &mut self,
        rpc_id: RpcId,
        (state, filter): (&State, Option<&str>),
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcStateGetResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        let response = if let Some(filter) = filter {
            let (json_state, filter) = optimize_filtered_state(state, filter)?;
            match filter.parse::<jsonpath_rust::JsonPathInst>() {
                Ok(filter) => {
                    let values = filter
                        .find_slice(&json_state, Default::default())
                        .into_iter()
                        .map(|p| (*p).clone())
                        .collect::<Vec<_>>();
                    Ok(if values.len() == 1 {
                        values[0].clone()
                    } else {
                        serde_json::Value::Array(values)
                    })
                }
                Err(err) => Err(RpcStateGetError::FilterError(err)),
            }
        } else {
            Ok(serde_json::to_value(state)?)
        };
        chan.send(response)
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }
    rpc_service_impl!(respond_status_get, RpcStatusGetResponse);

    rpc_service_impl!(respond_sync_stats_get, RpcSyncStatsGetResponse);
    rpc_service_impl!(respond_action_stats_get, RpcActionStatsGetResponse);
    rpc_service_impl!(
        respond_block_producer_stats_get,
        RpcBlockProducerStatsGetResponse
    );
    rpc_service_impl!(
        respond_message_progress_stats_get,
        RpcMessageProgressResponse
    );
    rpc_service_impl!(respond_peers_get, RpcPeersGetResponse);
    rpc_service_impl!(
        respond_p2p_connection_outgoing,
        RpcP2pConnectionOutgoingResponse
    );

    fn respond_p2p_connection_incoming_answer(
        &mut self,
        rpc_id: RpcId,
        response: P2pConnectionResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.get(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast_ref::<mpsc::Sender<RpcP2pConnectionIncomingResponse>>()
            .ok_or(RespondError::UnexpectedResponseType)?
            .clone();
        chan.try_send(RpcP2pConnectionIncomingResponse::Answer(response))
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_p2p_connection_incoming(
        &mut self,
        rpc_id: RpcId,
        response: Result<(), String>,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<mpsc::Sender<RpcP2pConnectionIncomingResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.try_send(RpcP2pConnectionIncomingResponse::Result(response))
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    rpc_service_impl!(
        respond_scan_state_summary_get,
        RpcScanStateSummaryGetResponse
    );
    rpc_service_impl!(respond_snark_pool_get, RpcSnarkPoolGetResponse);
    rpc_service_impl!(respond_snark_pool_job_get, RpcSnarkPoolJobGetResponse);
    rpc_service_impl!(respond_snarker_job_commit, RpcSnarkerJobCommitResponse);
    rpc_service_impl!(
        respond_snarker_job_spec,
        node::rpc::RpcSnarkerJobSpecResponse
    );
    rpc_service_impl!(
        respond_snarker_workers,
        node::rpc::RpcSnarkerWorkersResponse
    );
    rpc_service_impl!(
        respond_snarker_config_get,
        node::rpc::RpcSnarkerConfigGetResponse
    );
    rpc_service_impl!(respond_health_check, RpcHealthCheckResponse);
    rpc_service_impl!(respond_readiness_check, RpcReadinessCheckResponse);
    rpc_service_impl!(
        respond_discovery_routing_table,
        RpcDiscoveryRoutingTableResponse
    );
    rpc_service_impl!(
        respond_discovery_bootstrap_stats,
        RpcDiscoveryBoostrapStatsResponse
    );
    rpc_service_impl!(respond_transaction_pool, RpcTransactionPoolResponse);
    rpc_service_impl!(respond_ledger_accounts, RpcLedgerAccountsResponse);
    rpc_service_impl!(respond_transaction_inject, RpcTransactionInjectResponse);
    rpc_service_impl!(
        respond_transaction_inject_failed,
        RpcTransactionInjectFailure
    );
    rpc_service_impl!(
        respond_transition_frontier_commands,
        RpcTransitionFrontierUserCommandsResponse
    );
}

#[cfg(test)]
mod tests {
    use super::strip_root_field;

    #[test]
    fn strip_root_field_test() {
        for (filter, expected) in [
            ("$.field", Some("")),
            ("$['field']", Some("")),
            ("$.field.another", Some(".another")),
            ("$['field'].another", Some(".another")),
            ("$.another", None),
            ("$.field_1", None),
            ("$.fields", None),
        ] {
            let actual = strip_root_field(filter, "field");
            assert_eq!(actual, expected)
        }
    }
}

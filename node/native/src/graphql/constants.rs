use juniper::GraphQLObject;
use node::{
    rpc::{
        ConsensusTimeQuery, PeerConnectionStatus, RpcConsensusTimeGetResponse,
        RpcNodeStatusNetworkInfo, RpcPeerInfo, RpcRequest,
    },
    BuildEnv,
};
use openmina_core::{
    consensus::{ConsensusConstants, ConsensusTime},
    constants::ConstraintConstants,
};

use super::{Context, ConversionError, Error};

#[derive(Clone, Debug, Copy)]
pub(crate) struct GraphQLDaemonStatus;

#[juniper::graphql_object(context = Context)]
impl GraphQLDaemonStatus {
    async fn consensus_configuration(
        &self,
        context: &Context,
    ) -> juniper::FieldResult<GraphQLConsensusConfiguration> {
        let consensus_constants: ConsensusConstants = context
            .rpc_sender
            .oneshot_request(RpcRequest::ConsensusConstantsGet)
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;
        Ok(GraphQLConsensusConfiguration::from(consensus_constants))
    }

    async fn peers(&self, context: &Context) -> juniper::FieldResult<Vec<GraphQLRpcPeerInfo>> {
        let peers: Vec<RpcPeerInfo> = context
            .rpc_sender
            .oneshot_request(RpcRequest::PeersGet)
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        let connected_peers = peers
            .iter()
            .filter(|peer| matches!(peer.connection_status, PeerConnectionStatus::Connected))
            .map(GraphQLRpcPeerInfo::from)
            .collect();

        Ok(connected_peers)
    }

    async fn consensus_time_now(
        &self,
        context: &Context,
    ) -> juniper::FieldResult<GraphQLConsensusTime> {
        let consensus_time: RpcConsensusTimeGetResponse = context
            .rpc_sender
            .oneshot_request(RpcRequest::ConsensusTimeGet(ConsensusTimeQuery::Now))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        match consensus_time {
            Some(consensus_time) => Ok(GraphQLConsensusTime::from(consensus_time)),
            None => Err(juniper::FieldError::new(
                "No consensus time found",
                juniper::Value::Null,
            )),
        }
    }

    async fn consensus_time_best_tip(
        &self,
        context: &Context,
    ) -> juniper::FieldResult<GraphQLConsensusTime> {
        let consensus_time_res: RpcConsensusTimeGetResponse = context
            .rpc_sender
            .oneshot_request(RpcRequest::ConsensusTimeGet(ConsensusTimeQuery::BestTip))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        match consensus_time_res {
            Some(consensus_time) => Ok(GraphQLConsensusTime::from(consensus_time)),
            None => Err(juniper::FieldError::new(
                "No consensus time found",
                juniper::Value::Null,
            )),
        }
    }

    async fn consensus_mechanism(&self, _context: &Context) -> juniper::FieldResult<String> {
        Ok("proof_of_stake".to_string())
    }

    async fn blockchain_length(&self, context: &Context) -> juniper::FieldResult<Option<i32>> {
        let status = context.get_or_fetch_status().await;

        Ok(status.and_then(|status| {
            status
                .transition_frontier
                .best_tip
                .map(|block_summary| block_summary.height as i32)
        }))
    }

    async fn chain_id(&self, context: &Context) -> juniper::FieldResult<Option<String>> {
        let status = context.get_or_fetch_status().await;

        Ok(status.and_then(|status| status.chain_id))
    }

    async fn commit_id(&self, _context: &Context) -> juniper::FieldResult<String> {
        Ok(BuildEnv::get().git.commit_hash.to_string())
    }

    async fn global_slot_since_genesis_best_tip(
        &self,
        context: &Context,
    ) -> juniper::FieldResult<Option<i32>> {
        let best_tip = context.get_or_fetch_best_tip().await;
        Ok(best_tip.and_then(|best_tip| {
            println!("best_tip OK");
            best_tip.global_slot_since_genesis().try_into().ok()
        }))
    }

    async fn ledger_merkle_root(&self, context: &Context) -> juniper::FieldResult<Option<String>> {
        let best_tip = context.get_or_fetch_best_tip().await;

        Ok(best_tip.map(|best_tip| best_tip.merkle_root_hash().to_string()))
        // match best_tip {
        //     Some(best_tip) => {
        //         println!("best_tip_ledger_merkle_root {:?}", best_tip.merkle_root_hash());
        //         let ledger_status = context
        //             .get_or_fetch_ledger_status(best_tip.merkle_root_hash())
        //             .await;
        //         Ok(ledger_status
        //             .map(|ledger_status| ledger_status.best_tip_staged_ledger_hash.to_string()))
        //     }
        //     None => Ok(None),
        // }
    }

    async fn state_hash(&self, context: &Context) -> juniper::FieldResult<Option<String>> {
        let best_tip = context.get_or_fetch_best_tip().await;
        Ok(best_tip.map(|best_tip| best_tip.hash().to_string()))
    }

    async fn num_accounts(&self, context: &Context) -> juniper::FieldResult<Option<i32>> {
        let best_tip = context.get_or_fetch_best_tip().await;

        match best_tip {
            Some(best_tip) => {
                let ledger_status = context
                    .get_or_fetch_ledger_status(best_tip.merkle_root_hash())
                    .await;
                Ok(ledger_status.map(|ledger_status| ledger_status.num_accounts as i32))
            }
            None => Ok(None),
        }
    }

    async fn highest_unvalidated_block_length_received(
        &self,
        context: &Context,
    ) -> juniper::FieldResult<Option<i32>> {
        let status = context.get_or_fetch_status().await;
        Ok(status.and_then(|status| {
            status
                .transition_frontier
                .best_tip
                .map(|best_tip| best_tip.height as i32)
                .or_else(|| {
                    status
                        .transition_frontier
                        .sync
                        .target
                        .map(|target| target.height as i32)
                })
        }))
    }

    async fn highest_block_length_received(
        &self,
        context: &Context,
    ) -> juniper::FieldResult<Option<i32>> {
        let status = context.get_or_fetch_status().await;
        Ok(status.and_then(|status| {
            status
                .transition_frontier
                .best_tip
                .map(|best_tip| best_tip.height as i32)
                .or_else(|| {
                    status
                        .transition_frontier
                        .sync
                        .target
                        .map(|target| target.height as i32)
                })
        }))
    }

    async fn addrs_and_ports(
        &self,
        context: &Context,
    ) -> juniper::FieldResult<GraphQLAddrsAndPorts> {
        let status = context.get_or_fetch_status().await;

        match status {
            Some(status) => Ok(GraphQLAddrsAndPorts::from(&status.network_info)),
            None => Ok(Default::default()),
        }
    }

    async fn block_production_keys(&self, context: &Context) -> juniper::FieldResult<Vec<String>> {
        let status = context.get_or_fetch_status().await;
        Ok(status.map_or(vec![], |status| {
            status
                .block_producer
                .map_or(vec![], |key| vec![key.to_string()])
        }))
    }

    async fn coinbase_receiver(&self, context: &Context) -> juniper::FieldResult<Option<String>> {
        let status = context.get_or_fetch_status().await;
        Ok(status.and_then(|status| status.coinbase_receiver.map(|key| key.to_string())))
    }
}

#[derive(GraphQLObject, Clone, Debug)]
pub struct GraphQLAddrsAndPorts {
    pub bind_ip: String,
    pub external_ip: Option<String>,
    pub client_port: Option<i32>,
    pub libp2p_port: Option<i32>,
}

impl Default for GraphQLAddrsAndPorts {
    fn default() -> Self {
        Self {
            bind_ip: "0.0.0.0".to_string(),
            external_ip: None,
            client_port: None,
            libp2p_port: None,
        }
    }
}

impl From<&RpcNodeStatusNetworkInfo> for GraphQLAddrsAndPorts {
    fn from(network_info: &RpcNodeStatusNetworkInfo) -> Self {
        Self {
            bind_ip: network_info.bind_ip.clone(),
            external_ip: network_info.external_ip.clone(),
            client_port: network_info.client_port.map(|port| port.into()),
            libp2p_port: network_info.libp2p_port.map(|port| port.into()),
        }
    }
}

#[derive(GraphQLObject, Clone, Debug)]
pub struct GraphQLRpcPeerInfo {
    pub peer_id: String,
    pub best_tip: Option<String>,
    pub best_tip_height: Option<String>,
    pub best_tip_global_slot: Option<String>,
    pub best_tip_timestamp: Option<String>,
    pub connection_status: String,
    pub connecting_details: Option<String>,
    pub address: Option<String>,
    pub incoming: bool,
    pub is_libp2p: bool,
    pub time: String,
}

impl From<&RpcPeerInfo> for GraphQLRpcPeerInfo {
    fn from(peer: &RpcPeerInfo) -> Self {
        Self {
            peer_id: peer.peer_id.to_string(),
            best_tip: peer.best_tip.as_ref().map(|hash| hash.to_string()),
            best_tip_height: peer.best_tip_height.map(|height| height.to_string()),
            best_tip_global_slot: peer.best_tip_global_slot.map(|slot| slot.to_string()),
            best_tip_timestamp: peer
                .best_tip_timestamp
                .map(|timestamp| timestamp.to_string()),
            connection_status: peer.connection_status.to_string(),
            connecting_details: peer.connecting_details.clone(),
            address: peer.address.clone(),
            incoming: peer.incoming,
            is_libp2p: peer.is_libp2p,
            time: peer.time.to_string(),
        }
    }
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLGenesisConstants {
    pub genesis_timestamp: String,
    pub coinbase: String,
    pub account_creation_fee: String,
}

impl GraphQLGenesisConstants {
    pub fn try_new(
        constrain_constants: ConstraintConstants,
        consensus_constants: ConsensusConstants,
    ) -> Result<Self, ConversionError> {
        Ok(GraphQLGenesisConstants {
            genesis_timestamp: consensus_constants
                .human_readable_genesis_timestamp()
                .map_err(|e| ConversionError::Custom(e.to_string()))?,
            coinbase: constrain_constants.coinbase_amount.to_string(),
            account_creation_fee: constrain_constants.account_creation_fee.to_string(),
        })
    }
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLConsensusConfiguration {
    pub epoch_duration: i32,
    pub k: i32,
    pub slot_duration: i32,
    pub slots_per_epoch: i32,
}

impl From<ConsensusConstants> for GraphQLConsensusConfiguration {
    fn from(consensus_constants: ConsensusConstants) -> Self {
        GraphQLConsensusConfiguration {
            epoch_duration: consensus_constants.epoch_duration as i32,
            k: consensus_constants.k as i32,
            slot_duration: consensus_constants.slot_duration_ms as i32,
            slots_per_epoch: consensus_constants.slots_per_epoch as i32,
        }
    }
}

#[derive(GraphQLObject, Debug)]
pub struct GraphQLConsensusTime {
    pub start_time: String,
    pub end_time: String,
    pub epoch: String,
    pub global_slot: String,
    pub slot: String,
}

impl From<ConsensusTime> for GraphQLConsensusTime {
    fn from(consensus_time: ConsensusTime) -> Self {
        let start_time: u64 = consensus_time.start_time.into();
        let end_time: u64 = consensus_time.end_time.into();

        let start_time_ms = start_time.checked_div(1_000_000).expect("division by zero");
        let end_time_ms = end_time.checked_div(1_000_000).expect("division by zero");

        GraphQLConsensusTime {
            start_time: start_time_ms.to_string(),
            end_time: end_time_ms.to_string(),
            epoch: consensus_time.epoch.to_string(),
            global_slot: consensus_time.global_slot.to_string(),
            slot: consensus_time.slot.to_string(),
        }
    }
}

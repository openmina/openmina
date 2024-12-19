//! Basic connectivity tests.
//! Initial Joining:
//! * Ensure new nodes can discover peers and establish initial connections.
//! * Test how nodes handle scenarios when they are overwhelmed with too many connections or data requests.
//!
//! TODO(vlad9486):
//! Reconnection: Validate that nodes can reconnect after both intentional and unintentional disconnections.
//! Handling Latency: Nodes should remain connected and synchronize even under high latency conditions.
//! Intermittent Connections: Nodes should be resilient to sporadic network dropouts and still maintain synchronization.
//! Dynamic IP Handling: Nodes with frequently changing IP addresses should maintain stable connections.

pub mod multi_node;
pub mod record_replay;
pub mod simulation;
pub mod solo_node;

pub mod p2p;

mod driver;
pub use driver::*;

pub use crate::cluster::runner::*;

use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use crate::cluster::{Cluster, ClusterConfig};
use crate::scenario::{Scenario, ScenarioId, ScenarioStep};

use self::multi_node::basic_connectivity_initial_joining::MultiNodeBasicConnectivityInitialJoining;
use self::multi_node::basic_connectivity_peer_discovery::MultiNodeBasicConnectivityPeerDiscovery;
use self::multi_node::connection_discovery::RustNodeAsSeed as P2pConnectionDiscoveryRustNodeAsSeed;
use self::multi_node::connection_discovery::{
    OCamlToRust, OCamlToRustViaSeed, RustToOCaml, RustToOCamlViaSeed,
};
use self::multi_node::pubsub_advanced::MultiNodePubsubPropagateBlock;
use self::multi_node::sync_4_block_producers::MultiNodeSync4BlockProducers;
use self::multi_node::vrf_correct_ledgers::MultiNodeVrfGetCorrectLedgers;
use self::multi_node::vrf_correct_slots::MultiNodeVrfGetCorrectSlots;
use self::multi_node::vrf_epoch_bounds_correct_ledgers::MultiNodeVrfEpochBoundsCorrectLedger;
use self::multi_node::vrf_epoch_bounds_evaluation::MultiNodeVrfEpochBoundsEvaluation;
use self::p2p::basic_connection_handling::{
    AllNodesConnectionsAreSymmetric, MaxNumberOfPeersIncoming, MaxNumberOfPeersIs1,
    SeedConnectionsAreSymmetric, SimultaneousConnections,
};
use self::p2p::basic_incoming_connections::{
    AcceptIncomingConnection, AcceptMultipleIncomingConnections,
};
use self::p2p::basic_outgoing_connections::{
    ConnectToInitialPeers, ConnectToInitialPeersBecomeReady, ConnectToUnavailableInitialPeers,
    DontConnectToInitialPeerWithSameId, DontConnectToNodeWithSameId, DontConnectToSelfInitialPeer,
    MakeMultipleOutgoingConnections, MakeOutgoingConnection,
};
use self::p2p::kademlia::KademliaBootstrap;
use self::p2p::pubsub::P2pReceiveBlock;
use self::p2p::signaling::P2pSignaling;
use self::record_replay::block_production::RecordReplayBlockProduction;
use self::record_replay::bootstrap::RecordReplayBootstrap;
use self::simulation::small::SimulationSmall;
use self::simulation::small_forever_real_time::SimulationSmallForeverRealTime;
use self::solo_node::sync_to_genesis::SoloNodeSyncToGenesis;
use self::solo_node::sync_to_genesis_custom::SoloNodeSyncToGenesisCustom;
use self::solo_node::{
    basic_connectivity_accept_incoming::SoloNodeBasicConnectivityAcceptIncoming,
    basic_connectivity_initial_joining::SoloNodeBasicConnectivityInitialJoining,
    bootstrap::SoloNodeBootstrap, sync_root_snarked_ledger::SoloNodeSyncRootSnarkedLedger,
};

#[derive(EnumIter, EnumString, IntoStaticStr, derive_more::From, Clone, Copy)]
#[strum(serialize_all = "kebab-case")]
pub enum Scenarios {
    SoloNodeSyncToGenesis(SoloNodeSyncToGenesis),
    SoloNodeBootstrap(SoloNodeBootstrap),
    SoloNodeSyncToGenesisCustom(SoloNodeSyncToGenesisCustom),
    SoloNodeSyncRootSnarkedLedger(SoloNodeSyncRootSnarkedLedger),
    SoloNodeBasicConnectivityInitialJoining(SoloNodeBasicConnectivityInitialJoining),
    SoloNodeBasicConnectivityAcceptIncoming(SoloNodeBasicConnectivityAcceptIncoming),
    MultiNodeSync4BlockProducers(MultiNodeSync4BlockProducers),
    MultiNodeVrfGetCorrectLedgers(MultiNodeVrfGetCorrectLedgers),
    MultiNodeVrfGetCorrectSlots(MultiNodeVrfGetCorrectSlots),
    MultiNodeVrfEpochBoundsEvaluation(MultiNodeVrfEpochBoundsEvaluation),
    MultiNodeVrfEpochBoundsCorrectLedger(MultiNodeVrfEpochBoundsCorrectLedger),
    MultiNodeBasicConnectivityInitialJoining(MultiNodeBasicConnectivityInitialJoining),
    MultiNodeBasicConnectivityPeerDiscovery(MultiNodeBasicConnectivityPeerDiscovery),
    SimulationSmall(SimulationSmall),
    SimulationSmallForeverRealTime(SimulationSmallForeverRealTime),
    P2pReceiveBlock(P2pReceiveBlock),
    P2pSignaling(P2pSignaling),
    P2pConnectionDiscoveryRustNodeAsSeed(P2pConnectionDiscoveryRustNodeAsSeed),
    MultiNodePubsubPropagateBlock(MultiNodePubsubPropagateBlock),
    RecordReplayBootstrap(RecordReplayBootstrap),
    RecordReplayBlockProduction(RecordReplayBlockProduction),

    RustToOCaml(RustToOCaml),
    OCamlToRust(OCamlToRust),
    OCamlToRustViaSeed(OCamlToRustViaSeed),
    RustToOCamlViaSeed(RustToOCamlViaSeed),
    KademliaBootstrap(KademliaBootstrap),
    AcceptIncomingConnection(AcceptIncomingConnection),
    MakeOutgoingConnection(MakeOutgoingConnection),
    AcceptMultipleIncomingConnections(AcceptMultipleIncomingConnections),
    MakeMultipleOutgoingConnections(MakeMultipleOutgoingConnections),
    DontConnectToNodeWithSameId(DontConnectToNodeWithSameId),
    DontConnectToInitialPeerWithSameId(DontConnectToInitialPeerWithSameId),
    DontConnectToSelfInitialPeer(DontConnectToSelfInitialPeer),
    SimultaneousConnections(SimultaneousConnections),
    ConnectToInitialPeers(ConnectToInitialPeers),
    ConnectToUnavailableInitialPeers(ConnectToUnavailableInitialPeers),
    AllNodesConnectionsAreSymmetric(AllNodesConnectionsAreSymmetric),
    ConnectToInitialPeersBecomeReady(ConnectToInitialPeersBecomeReady),
    SeedConnectionsAreSymmetric(SeedConnectionsAreSymmetric),
    MaxNumberOfPeersIncoming(MaxNumberOfPeersIncoming),
    MaxNumberOfPeersIs1(MaxNumberOfPeersIs1),
}

impl Scenarios {
    // Turn off global test
    pub fn iter() -> impl IntoIterator<Item = Scenarios> {
        <Self as strum::IntoEnumIterator>::iter().filter(|s| !s.skip())
    }

    pub fn find_by_name(name: &str) -> Option<Self> {
        <Self as strum::IntoEnumIterator>::iter().find(|v| v.to_str() == name)
    }

    fn skip(&self) -> bool {
        match self {
            Self::SoloNodeSyncToGenesis(_) => true,
            Self::SoloNodeSyncToGenesisCustom(_) => true,
            Self::SoloNodeBasicConnectivityAcceptIncoming(_) => cfg!(feature = "p2p-webrtc"),
            Self::MultiNodeBasicConnectivityPeerDiscovery(_) => cfg!(feature = "p2p-webrtc"),
            Self::SimulationSmall(_) => true,
            Self::SimulationSmallForeverRealTime(_) => true,
            Self::MultiNodePubsubPropagateBlock(_) => true, // in progress
            Self::P2pSignaling(_) => cfg!(feature = "p2p-webrtc"),
            _ => false,
        }
    }

    pub fn id(self) -> ScenarioId {
        self.into()
    }

    pub fn to_str(self) -> &'static str {
        self.into()
    }

    pub fn parent(self) -> Option<Self> {
        match self {
            Self::MultiNodeSync4BlockProducers(_) => Some(SoloNodeSyncToGenesis.into()),
            Self::MultiNodeVrfGetCorrectLedgers(_) => Some(SoloNodeSyncToGenesisCustom.into()),
            Self::MultiNodeVrfGetCorrectSlots(_) => Some(SoloNodeSyncToGenesisCustom.into()),
            Self::MultiNodeVrfEpochBoundsEvaluation(_) => Some(SoloNodeSyncToGenesisCustom.into()),
            Self::MultiNodeVrfEpochBoundsCorrectLedger(_) => {
                Some(SoloNodeSyncToGenesisCustom.into())
            }
            _ => None,
        }
    }

    pub fn parent_id(self) -> Option<ScenarioId> {
        self.parent().map(Self::id)
    }

    pub fn description(self) -> &'static str {
        use documented::Documented;
        match self {
            Self::SoloNodeSyncToGenesis(_) => SoloNodeSyncToGenesis::DOCS,
            Self::SoloNodeBootstrap(_) => SoloNodeBootstrap::DOCS,
            Self::SoloNodeSyncToGenesisCustom(_) => SoloNodeSyncToGenesis::DOCS,
            Self::SoloNodeSyncRootSnarkedLedger(_) => SoloNodeSyncRootSnarkedLedger::DOCS,
            Self::SoloNodeBasicConnectivityInitialJoining(_) => {
                SoloNodeBasicConnectivityInitialJoining::DOCS
            }
            Self::SoloNodeBasicConnectivityAcceptIncoming(_) => {
                SoloNodeBasicConnectivityAcceptIncoming::DOCS
            }
            Self::MultiNodeSync4BlockProducers(_) => MultiNodeSync4BlockProducers::DOCS,
            Self::MultiNodeVrfGetCorrectLedgers(_) => MultiNodeVrfGetCorrectLedgers::DOCS,
            Self::MultiNodeVrfGetCorrectSlots(_) => MultiNodeVrfGetCorrectSlots::DOCS,
            Self::MultiNodeVrfEpochBoundsEvaluation(_) => MultiNodeVrfEpochBoundsEvaluation::DOCS,
            Self::MultiNodeVrfEpochBoundsCorrectLedger(_) => {
                MultiNodeVrfEpochBoundsCorrectLedger::DOCS
            }
            Self::MultiNodeBasicConnectivityInitialJoining(_) => {
                MultiNodeBasicConnectivityInitialJoining::DOCS
            }
            Self::MultiNodeBasicConnectivityPeerDiscovery(_) => {
                MultiNodeBasicConnectivityPeerDiscovery::DOCS
            }
            Self::SimulationSmall(_) => SimulationSmall::DOCS,
            Self::SimulationSmallForeverRealTime(_) => SimulationSmallForeverRealTime::DOCS,
            Self::P2pReceiveBlock(_) => P2pReceiveBlock::DOCS,
            Self::P2pSignaling(_) => P2pSignaling::DOCS,
            Self::P2pConnectionDiscoveryRustNodeAsSeed(_) => {
                P2pConnectionDiscoveryRustNodeAsSeed::DOCS
            }
            Self::MultiNodePubsubPropagateBlock(_) => MultiNodePubsubPropagateBlock::DOCS,
            Self::RecordReplayBootstrap(_) => RecordReplayBootstrap::DOCS,
            Self::RecordReplayBlockProduction(_) => RecordReplayBlockProduction::DOCS,

            Self::RustToOCaml(_) => RustToOCaml::DOCS,
            Self::OCamlToRust(_) => OCamlToRust::DOCS,
            Self::OCamlToRustViaSeed(_) => OCamlToRustViaSeed::DOCS,
            Self::RustToOCamlViaSeed(_) => RustToOCamlViaSeed::DOCS,
            Self::KademliaBootstrap(_) => KademliaBootstrap::DOCS,
            Self::AcceptIncomingConnection(_) => AcceptIncomingConnection::DOCS,
            Self::MakeOutgoingConnection(_) => MakeOutgoingConnection::DOCS,
            Self::AcceptMultipleIncomingConnections(_) => AcceptMultipleIncomingConnections::DOCS,
            Self::MakeMultipleOutgoingConnections(_) => MakeMultipleOutgoingConnections::DOCS,
            Self::DontConnectToNodeWithSameId(_) => DontConnectToNodeWithSameId::DOCS,
            Self::DontConnectToInitialPeerWithSameId(_) => DontConnectToInitialPeerWithSameId::DOCS,
            Self::DontConnectToSelfInitialPeer(_) => DontConnectToSelfInitialPeer::DOCS,
            Self::SimultaneousConnections(_) => SimultaneousConnections::DOCS,
            Self::ConnectToInitialPeers(_) => ConnectToInitialPeers::DOCS,
            Self::ConnectToUnavailableInitialPeers(_) => ConnectToUnavailableInitialPeers::DOCS,
            Self::AllNodesConnectionsAreSymmetric(_) => AllNodesConnectionsAreSymmetric::DOCS,
            Self::ConnectToInitialPeersBecomeReady(_) => ConnectToInitialPeersBecomeReady::DOCS,
            Self::SeedConnectionsAreSymmetric(_) => SeedConnectionsAreSymmetric::DOCS,
            Self::MaxNumberOfPeersIncoming(_) => MaxNumberOfPeersIncoming::DOCS,
            Self::MaxNumberOfPeersIs1(_) => MaxNumberOfPeersIs1::DOCS,
        }
    }

    pub fn default_cluster_config(self) -> Result<ClusterConfig, anyhow::Error> {
        let config = ClusterConfig::new(None)
            .map_err(|err| anyhow::anyhow!("failed to create cluster configuration: {err}"))?;

        match self {
            Self::P2pSignaling(v) => v.default_cluster_config(config),
            _ => Ok(config),
        }
    }

    pub fn blank_scenario(self) -> Scenario {
        let mut scenario = Scenario::new(self.id(), self.parent_id());
        scenario.set_description(self.description().to_owned());
        scenario.info.nodes = Vec::new();

        scenario
    }

    async fn run<F>(self, cluster: &mut Cluster, add_step: F)
    where
        F: Send + FnMut(&ScenarioStep),
    {
        let runner = ClusterRunner::new(cluster, add_step);
        match self {
            Self::SoloNodeSyncToGenesis(v) => v.run(runner).await,
            Self::SoloNodeBootstrap(v) => v.run(runner).await,
            Self::SoloNodeSyncToGenesisCustom(v) => v.run(runner).await,
            Self::SoloNodeSyncRootSnarkedLedger(v) => v.run(runner).await,
            Self::SoloNodeBasicConnectivityInitialJoining(v) => v.run(runner).await,
            Self::SoloNodeBasicConnectivityAcceptIncoming(v) => v.run(runner).await,
            Self::MultiNodeSync4BlockProducers(v) => v.run(runner).await,
            Self::MultiNodeVrfGetCorrectLedgers(v) => v.run(runner).await,
            Self::MultiNodeVrfGetCorrectSlots(v) => v.run(runner).await,
            Self::MultiNodeVrfEpochBoundsEvaluation(v) => v.run(runner).await,
            Self::MultiNodeVrfEpochBoundsCorrectLedger(v) => v.run(runner).await,
            Self::MultiNodeBasicConnectivityInitialJoining(v) => v.run(runner).await,
            Self::MultiNodeBasicConnectivityPeerDiscovery(v) => v.run(runner).await,
            Self::SimulationSmall(v) => v.run(runner).await,
            Self::SimulationSmallForeverRealTime(v) => v.run(runner).await,
            Self::P2pReceiveBlock(v) => v.run(runner).await,
            Self::P2pSignaling(v) => v.run(runner).await,
            Self::P2pConnectionDiscoveryRustNodeAsSeed(v) => v.run(runner).await,
            Self::MultiNodePubsubPropagateBlock(v) => v.run(runner).await,
            Self::RecordReplayBootstrap(v) => v.run(runner).await,
            Self::RecordReplayBlockProduction(v) => v.run(runner).await,

            Self::RustToOCaml(v) => v.run(runner).await,
            Self::OCamlToRust(v) => v.run(runner).await,
            Self::OCamlToRustViaSeed(v) => v.run(runner).await,
            Self::RustToOCamlViaSeed(v) => v.run(runner).await,
            Self::KademliaBootstrap(v) => v.run(runner).await,
            Self::AcceptIncomingConnection(v) => v.run(runner).await,
            Self::MakeOutgoingConnection(v) => v.run(runner).await,
            Self::AcceptMultipleIncomingConnections(v) => v.run(runner).await,
            Self::MakeMultipleOutgoingConnections(v) => v.run(runner).await,
            Self::DontConnectToNodeWithSameId(v) => v.run(runner).await,
            Self::DontConnectToInitialPeerWithSameId(v) => v.run(runner).await,
            Self::DontConnectToSelfInitialPeer(v) => v.run(runner).await,
            Self::SimultaneousConnections(v) => v.run(runner).await,
            Self::ConnectToInitialPeers(v) => v.run(runner).await,
            Self::ConnectToUnavailableInitialPeers(v) => v.run(runner).await,
            Self::AllNodesConnectionsAreSymmetric(v) => v.run(runner).await,
            Self::ConnectToInitialPeersBecomeReady(v) => v.run(runner).await,
            Self::SeedConnectionsAreSymmetric(v) => v.run(runner).await,
            Self::MaxNumberOfPeersIncoming(v) => v.run(runner).await,
            Self::MaxNumberOfPeersIs1(v) => v.run(runner).await,
        }
    }

    pub async fn run_and_save(self, cluster: &mut Cluster) {
        struct ScenarioSaveOnExit(Scenario);

        impl Drop for ScenarioSaveOnExit {
            fn drop(&mut self) {
                let info = self.0.info.clone();
                let steps = std::mem::take(&mut self.0.steps);
                let scenario = Scenario { info, steps };

                eprintln!("saving scenario({}) before exit...", scenario.info.id);
                if let Err(err) = scenario.save_sync() {
                    eprintln!(
                        "failed to save scenario({})! error: {}",
                        scenario.info.id, err
                    );
                }
            }
        }

        eprintln!("run_and_save: {}", self.to_str());
        let mut scenario = ScenarioSaveOnExit(self.blank_scenario());
        self.run(cluster, |step| scenario.0.add_step(step.clone()).unwrap())
            .await;
        // drop to save it.
        let _ = scenario;
    }

    pub async fn run_only(self, cluster: &mut Cluster) {
        eprintln!("run_only: {}", self.to_str());
        self.run(cluster, |_| {}).await
    }

    async fn build_cluster_and_run_parents(self, config: ClusterConfig) -> Cluster {
        let mut parents = std::iter::repeat(())
            .scan(self.parent(), |parent, _| {
                let cur_parent = parent.take();
                *parent = cur_parent.and_then(|p| p.parent());
                cur_parent
            })
            .collect::<Vec<_>>();

        let mut cluster = Cluster::new(config);
        while let Some(scenario) = parents.pop() {
            scenario.run_only(&mut cluster).await;
        }

        cluster
    }

    pub async fn run_and_save_from_scratch(self, config: ClusterConfig) {
        let mut cluster = self.build_cluster_and_run_parents(config).await;
        self.run_and_save(&mut cluster).await;
    }

    pub async fn run_only_from_scratch(self, config: ClusterConfig) {
        let mut cluster = self.build_cluster_and_run_parents(config).await;
        self.run_only(&mut cluster).await;
    }
}

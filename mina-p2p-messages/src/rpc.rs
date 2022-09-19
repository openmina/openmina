///! Mina RPC methods

use std::collections::BTreeMap;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::{string::CharString, versioned::{Versioned, Ver}, core::InetAddrV1Binable};
use crate::common::*;
use crate::core;
use crate::rpc_kernel::*;
use crate::v1::{
    ConsensusProofOfStakeDataConsensusStateValueStableV1Binable,
    //TransactionSnarkScanStateStableV1Binable,
    MinaBasePendingCoinbaseStableV1Binable,
    MinaBaseSparseLedgerStableV1Binable,
    MinaBaseSyncLedgerAnswerStableV1Binable,
    MinaBaseSyncLedgerQueryStableV1Binable,
    MinaBlockExternalTransitionRawVersionedStableV1Binable,
    MinaStateProtocolStateValueStableV1Binable,
    NetworkPeerPeerIdStableV1Binable,
    PublicKeyCompressedStableV1Binable,
    SyncStatusTStableV1Binable,
    TransactionSnarkScanStateStableV1Binable,
    TrustSystemPeerStatusStableV1Binable,
};

macro_rules! mina_rpc {
    ($name:ident, $tag:literal, $version:literal, $query:ty, $response:ty $(,)?) => {
        pub struct $name;
        impl crate::rpc_kernel::RpcMethod for $name {
            const NAME: &'static str = $tag;
            const VERSION: crate::versioned::Ver = $version;
            type Query = $query;
            type Response = $response;
        }
    };
}

mina_rpc!(
    VersionedRpcMenuV1,
    "__Versioned_rpc.Menu",
    1,
    (),
    Vec<(CharString, Ver)>
);

mina_rpc!(
    GetSomeInitialPeersV1,
    "get_some_initial_peers",
    1,
    (),
    Vec<NetworkPeerPeerIdStableV1Binable>
);

pub type GetStagedLedgerAuxAndPendingCoinbasesAtHashV1Response = Option<(
    TransactionSnarkScanStateStableV1Binable,
    LedgerHashV1Binable,
    MinaBasePendingCoinbaseStableV1Binable,
    Vec<MinaStateProtocolStateValueStableV1Binable>,
)>;

mina_rpc!(
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV1,
    "get_staged_ledger_aux_and_pending_coinbases_at_hash",
    1,
    StateHashV1Binable,
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV1Response,
);

mina_rpc!(
    AnswerSyncLedgerQueryV1,
    "answer_sync_ledger_query",
    1,
    (LedgerHashV1Binable, MinaBaseSyncLedgerQueryStableV1Binable),
    RpcResult<MinaBaseSyncLedgerAnswerStableV1Binable, core::Error>
);
mina_rpc!(
    GetTransitionChainV1,
    "get_transition_chain",
    1,
    Vec<StateHashV1Binable>,
    Option<Vec<MinaBlockExternalTransitionRawVersionedStableV1Binable>>
);

pub type GetTransitionChainProofV1Response =
    Option<(StateHashV1Binable, Vec<StateBodyHashV1Binable>)>;
mina_rpc!(
    GetTransitionChainProofV1,
    "get_transition_chain_proof",
    1,
    StateHashV1Binable,
    GetTransitionChainProofV1Response,
);

mina_rpc!(
    GetTransitionKnowledgeV1,
    "Get_transition_knowledge",
    1,
    (),
    Vec<StateHashV1Binable>
);

// pub struct ConsensusDataConsensusStateValue;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct WithHashV1<A, H> {
    data: A,
    hash: H,
}
pub type WithHashV1Binable<A, H> = Versioned<WithHashV1<A, H>, 1>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProofCarryingDataV1<A, B> {
    data: A,
    proof: B,
}
pub type ProofCarryingDataWithHashV1Binable<A, B> = Versioned<ProofCarryingDataV1<A, B>, 1>;

pub type GetAncestryV1Query = WithHashV1Binable<
    ConsensusProofOfStakeDataConsensusStateValueStableV1Binable,
    StateHashV1Binable,
>;
pub type GetAncestryV1Response = Option<
    ProofCarryingDataWithHashV1Binable<
        MinaBlockExternalTransitionRawVersionedStableV1Binable,
        (
            Vec<StateBodyHashV1Binable>,
            MinaBlockExternalTransitionRawVersionedStableV1Binable,
        ),
    >,
>;
mina_rpc!(
    GetAncestryV1,
    "get_ancestry",
    1,
    GetAncestryV1Query,
    GetAncestryV1Response,
);

mina_rpc!(BanNotifyV1, "ban_notify", 1, core::Time, ());

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProofCarryingDataStableV1<A, B> {
    data: A,
    proof: B,
}
pub type ProofCarryingDataStableV1Binable<A, B> = Versioned<ProofCarryingDataStableV1<A, B>, 1>;
pub type GetBestTipV1Response = Option<
    ProofCarryingDataStableV1Binable<
        MinaBlockExternalTransitionRawVersionedStableV1Binable,
        (
            Vec<LedgerHashV1Binable>,
            MinaBlockExternalTransitionRawVersionedStableV1Binable,
        ),
    >,
>;
mina_rpc!(GetBestTipV1, "get_best_tip", 1, (), GetBestTipV1Response);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NodeStatusV1 {
    node_ip_addr: InetAddrV1Binable,
    node_peer_id: NetworkPeerPeerIdStableV1Binable,
    sync_status: SyncStatusTStableV1Binable,
    peers: Vec<NetworkPeerPeerIdStableV1Binable>,
    block_producers: Vec<PublicKeyCompressedStableV1Binable>,
    ban_statuses: Vec<(
        NetworkPeerPeerIdStableV1Binable,
        TrustSystemPeerStatusStableV1Binable,
    )>,
    k_block_hashes_and_timestamps: Vec<(StateHashV1Binable, CharString)>,
    git_commit: CharString,
    uptime_minutes: i32,
}
mina_rpc!(GetNodeStatusV1, "get_node_status", 1, (), RpcResult<NodeStatusV1, core::Error>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NodeStatusV2 {
    node_ip_addr: InetAddrV1Binable,
    node_peer_id: NetworkPeerPeerIdStableV1Binable,
    sync_status: SyncStatusTStableV1Binable,
    peers: Vec<NetworkPeerPeerIdStableV1Binable>,
    block_producers: Vec<PublicKeyCompressedStableV1Binable>,
    protocol_state_hash: StateHashV1Binable,
    ban_statuses: Vec<(
        NetworkPeerPeerIdStableV1Binable,
        TrustSystemPeerStatusStableV1Binable,
    )>,
    k_block_hashes_and_timestamps: Vec<(StateHashV1Binable, CharString)>,
    git_commit: CharString,
    uptime_minutes: i32,
    block_height_opt: Option<i32>,
}
mina_rpc!(GetNodeStatusV2, "get_node_status", 2, (), RpcResult<NodeStatusV2, core::Error>);

mina_rpc!(GetEpochLedgerV1, "get_epoch_ledger", 1, LedgerHashV1Binable, RpcResult<MinaBaseSparseLedgerStableV1Binable, CharString>);

/// Registry for uniformly JSONifying RPC payload data.
///
/// ```
/// let r = mina_p2p_messages::JSONifyPayloadRegistry::new();
/// let mut d = &b"\x01\x00"[..];
/// let jsonifier = r.get("get_some_initial_peers", 1).unwrap();
/// let json = jsonifier.read_query(&mut d).unwrap();
/// ```
pub struct JSONifyPayloadRegistry {
    table: BTreeMap<(&'static str, Ver), Box<dyn JSONinifyPayloadReader>>,
}

impl JSONifyPayloadRegistry {
    pub fn new() -> Self {
        let mut this = Self {
            table: BTreeMap::new(),
        };
        this.insert(VersionedRpcMenuV1);
        this.insert(GetSomeInitialPeersV1);
        this.insert(GetStagedLedgerAuxAndPendingCoinbasesAtHashV1);
        this.insert(AnswerSyncLedgerQueryV1);
        this.insert(GetTransitionChainV1);
        this.insert(GetTransitionChainProofV1);
        this.insert(GetTransitionKnowledgeV1);
        this.insert(BanNotifyV1);
        this.insert(GetAncestryV1);
        this.insert(GetBestTipV1);
        this.insert(GetNodeStatusV1);
        this.insert(GetNodeStatusV2);
        this.insert(GetEpochLedgerV1);
        this
    }

    pub fn get<'a, 'b: 'a>(
        &'a self,
        name: &'b str,
        version: Ver,
    ) -> Option<&'a dyn JSONinifyPayloadReader> {
        self.table.get(&(name, version)).map(Box::as_ref)
    }

    fn insert<T>(&mut self, t: T)
    where
        T: RpcMethod + 'static,
        T::Query: Serialize,
        T::Response: Serialize,
    {
        self.table.insert((T::NAME, T::VERSION), Box::new(t));
    }
}

#[cfg(test)]
mod tests {
    use crate::JSONifyPayloadRegistry;

    #[test]
    fn jsonify_registry_content() {
        let r = JSONifyPayloadRegistry::new();
        for (name, version) in [
            ("__Versioned_rpc.Menu", 1),
            ("get_some_initial_peers", 1),
            ("get_staged_ledger_aux_and_pending_coinbases_at_hash", 1),
            ("answer_sync_ledger_query", 1),
            ("get_transition_chain", 1),
            ("get_transition_chain_proof", 1),
            ("Get_transition_knowledge", 1),
            ("get_ancestry", 1),
            ("ban_notify", 1),
            ("get_best_tip", 1),
            ("get_node_status", 1),
            ("get_node_status", 2),
            ("get_epoch_ledger", 1),
        ] {
            assert!(r.get(name, version).is_some());
        }
    }

    #[test]
    fn jsonify_registry_query() {
        let r = JSONifyPayloadRegistry::new();
        let payload =
            hex::decode("220101e7dd9b0d45abb2e4dec2c5d22e1f1bd8ae5133047914209a0229e90a62ecfb0e")
                .unwrap();
        let mut ptr = payload.as_slice();
        let jsonify = r.get("get_transition_chain", 1).unwrap();
        let json = jsonify.read_query(&mut ptr).unwrap();
        let expected =
            serde_json::json!(["e7dd9b0d45abb2e4dec2c5d22e1f1bd8ae5133047914209a0229e90a62ecfb0e"]);
        assert_eq!(json, expected);
    }
}

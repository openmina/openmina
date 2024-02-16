///! Mina RPC methods
use std::collections::BTreeMap;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::core;
use crate::rpc_kernel::*;
use crate::{
    core::InetAddrV1Versioned,
    string::CharString,
    v2::MinaBaseSparseLedgerBaseStableV2,
    versioned::{Ver, Versioned},
};
use crate::{v1, v2};

macro_rules! mina_rpc {
    ($name:ident, $tag:literal, $version:literal, $query:ty, $response:ty $(,)?) => {
        #[derive(Debug)]
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
    Vec<v1::NetworkPeerPeerIdStableV1Versioned>
);

mina_rpc!(
    GetSomeInitialPeersV1ForV2,
    "get_some_initial_peers",
    1,
    (),
    Vec<v2::NetworkPeerPeerStableV1>
);

pub type GetStagedLedgerAuxAndPendingCoinbasesAtHashV1Response = Option<(
    v1::TransactionSnarkScanStateStableV1Versioned,
    LedgerHashV1Versioned,
    v1::MinaBasePendingCoinbaseStableV1Versioned,
    Vec<v1::MinaStateProtocolStateValueStableV1Versioned>,
)>;

mina_rpc!(
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV1,
    "get_staged_ledger_aux_and_pending_coinbases_at_hash",
    1,
    StateHashV1Versioned,
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV1Response,
);

pub type GetStagedLedgerAuxAndPendingCoinbasesAtHashV2Response = Option<(
    v2::TransactionSnarkScanStateStableV2,
    LedgerHashV1,
    v2::MinaBasePendingCoinbaseStableV2,
    Vec<v2::MinaStateProtocolStateValueStableV2>,
)>;

mina_rpc!(
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV2,
    "get_staged_ledger_aux_and_pending_coinbases_at_hash",
    2,
    StateHashV1,
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV2Response,
);

mina_rpc!(
    AnswerSyncLedgerQueryV1,
    "answer_sync_ledger_query",
    1,
    (LedgerHashV1Versioned, v1::MinaBaseSyncLedgerQueryStableV1Versioned),
    RpcResult<v1::MinaBaseSyncLedgerAnswerStableV1Versioned, core::Error>
);

mina_rpc!(
    AnswerSyncLedgerQueryV2,
    "answer_sync_ledger_query",
    3,
    (LedgerHashV1, v2::MinaLedgerSyncLedgerQueryStableV1),
    RpcResult<v2::MinaLedgerSyncLedgerAnswerStableV2, core::Error>
);

mina_rpc!(
    GetTransitionChainV1,
    "get_transition_chain",
    1,
    Vec<StateHashV1Versioned>,
    Option<Vec<v1::MinaBlockExternalTransitionRawVersionedStableV1Versioned>>
);

mina_rpc!(
    GetTransitionChainV2,
    "get_transition_chain",
    2,
    Vec<StateHashV1>,
    Option<Vec<v2::MinaBlockBlockStableV2>>
);

pub type GetTransitionChainProofV1Response =
    Option<(StateHashV1Versioned, Vec<StateBodyHashV1Versioned>)>;
mina_rpc!(
    GetTransitionChainProofV1,
    "get_transition_chain_proof",
    1,
    StateHashV1Versioned,
    GetTransitionChainProofV1Response,
);

pub type GetTransitionChainProofV1ForV2Response = Option<(StateHashV1, Vec<StateBodyHashV1>)>;
mina_rpc!(
    GetTransitionChainProofV1ForV2,
    "get_transition_chain_proof",
    1,
    StateHashV1,
    GetTransitionChainProofV1ForV2Response,
);

mina_rpc!(
    GetTransitionKnowledgeV1,
    "Get_transition_knowledge",
    1,
    (),
    Vec<StateHashV1Versioned>
);

mina_rpc!(
    GetTransitionKnowledgeV1ForV2,
    "Get_transition_knowledge",
    1,
    (),
    Vec<StateHashV1>
);

// pub struct ConsensusDataConsensusStateValue;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct WithHashV1<A, H> {
    pub data: A,
    pub hash: H,
}
pub type WithHashV1Versioned<A, H> = Versioned<WithHashV1<A, H>, 1>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProofCarryingDataWithHashV1<A, B> {
    pub data: A,
    pub proof: B,
}
pub type ProofCarryingDataWithHashV1Versioned<A, B> =
    Versioned<ProofCarryingDataWithHashV1<A, B>, 1>;

pub type GetAncestryV1Query = WithHashV1Versioned<
    v1::ConsensusProofOfStakeDataConsensusStateValueStableV1Versioned,
    StateHashV1Versioned,
>;
pub type GetAncestryV1Response = Option<
    ProofCarryingDataWithHashV1Versioned<
        v1::MinaBlockExternalTransitionRawVersionedStableV1Versioned,
        (
            Vec<StateBodyHashV1Versioned>,
            v1::MinaBlockExternalTransitionRawVersionedStableV1Versioned,
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

pub type GetAncestryV2Query =
    WithHashV1<v2::ConsensusProofOfStakeDataConsensusStateValueStableV2, StateHashV1>;
pub type GetAncestryV2Response = Option<
    ProofCarryingDataWithHashV1<
        v2::MinaBlockBlockStableV2,
        (Vec<StateBodyHashV1>, v2::MinaBlockBlockStableV2),
    >,
>;
mina_rpc!(
    GetAncestryV2,
    "get_ancestry",
    2,
    GetAncestryV2Query,
    GetAncestryV2Response,
);

mina_rpc!(BanNotifyV1, "ban_notify", 1, core::Time, ());

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProofCarryingDataStableV1<A, B> {
    pub data: A,
    pub proof: B,
}
pub type ProofCarryingDataStableV1Versioned<A, B> = Versioned<ProofCarryingDataStableV1<A, B>, 1>;
pub type GetBestTipV1Response = Option<
    ProofCarryingDataStableV1Versioned<
        v1::MinaBlockExternalTransitionRawVersionedStableV1Versioned,
        (
            Vec<LedgerHashV1Versioned>,
            v1::MinaBlockExternalTransitionRawVersionedStableV1Versioned,
        ),
    >,
>;
pub type GetBestTipV2Response = Option<
    ProofCarryingDataStableV1<
        v2::MinaBlockBlockStableV2,
        (
            Vec<v2::MinaBaseStateBodyHashStableV1>,
            v2::MinaBlockBlockStableV2,
        ),
    >,
>;
mina_rpc!(GetBestTipV1, "get_best_tip", 1, (), GetBestTipV1Response);
mina_rpc!(GetBestTipV2, "get_best_tip", 2, (), GetBestTipV2Response);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NodeStatusV1 {
    node_ip_addr: InetAddrV1Versioned,
    node_peer_id: v1::NetworkPeerPeerIdStableV1Versioned,
    sync_status: v1::SyncStatusTStableV1Versioned,
    peers: Vec<v1::NetworkPeerPeerIdStableV1Versioned>,
    block_producers: Vec<v1::PublicKeyCompressedStableV1Versioned>,
    ban_statuses: Vec<(
        v1::NetworkPeerPeerIdStableV1Versioned,
        v1::TrustSystemPeerStatusStableV1Versioned,
    )>,
    k_block_hashes_and_timestamps: Vec<(StateHashV1Versioned, CharString)>,
    git_commit: CharString,
    uptime_minutes: i32,
}
mina_rpc!(GetNodeStatusV1, "get_node_status", 1, (), RpcResult<NodeStatusV1, core::Error>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NodeStatusV2 {
    node_ip_addr: InetAddrV1Versioned,
    node_peer_id: v1::NetworkPeerPeerIdStableV1Versioned,
    sync_status: v1::SyncStatusTStableV1Versioned,
    peers: Vec<v1::NetworkPeerPeerIdStableV1Versioned>,
    block_producers: Vec<v1::PublicKeyCompressedStableV1Versioned>,
    protocol_state_hash: StateHashV1Versioned,
    ban_statuses: Vec<(
        v1::NetworkPeerPeerIdStableV1Versioned,
        v1::TrustSystemPeerStatusStableV1Versioned,
    )>,
    k_block_hashes_and_timestamps: Vec<(StateHashV1Versioned, CharString)>,
    git_commit: CharString,
    uptime_minutes: i32,
    block_height_opt: Option<i32>,
}
mina_rpc!(GetNodeStatusV2, "get_node_status", 2, (), RpcResult<NodeStatusV2, core::Error>);

mina_rpc!(GetEpochLedgerV1, "get_epoch_ledger", 1, LedgerHashV1Versioned, RpcResult<v1::MinaBaseSparseLedgerStableV1Versioned, CharString>);

mina_rpc!(GetEpochLedgerV2, "get_epoch_ledger", 2, LedgerHashV1, RpcResult<MinaBaseSparseLedgerBaseStableV2, CharString>);

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
    #[deprecated = "Use `[v1]` or `[v2]` methods instead."]
    pub fn new() -> Self {
        Self::v1()
    }

    /// Creates registry with Mina V1 specific RPC methods.
    pub fn v1() -> Self {
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

    /// Creates registry with Mina V2 specific RPC methods.
    pub fn v2() -> Self {
        let mut this = Self {
            table: BTreeMap::new(),
        };
        this.insert(VersionedRpcMenuV1);
        this.insert(GetSomeInitialPeersV1ForV2);
        this.insert(GetStagedLedgerAuxAndPendingCoinbasesAtHashV2);
        this.insert(AnswerSyncLedgerQueryV2);
        this.insert(GetTransitionChainV2);
        this.insert(GetTransitionChainProofV1ForV2);
        this.insert(GetTransitionKnowledgeV1ForV2);
        this.insert(BanNotifyV1);
        this.insert(GetAncestryV2);
        this.insert(GetBestTipV2);
        this.insert(GetNodeStatusV1);
        this.insert(GetNodeStatusV2);
        this.insert(GetEpochLedgerV2);
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
    fn jsonify_registry_content_v1() {
        let r = JSONifyPayloadRegistry::v1();
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
    fn jsonify_registry_content_v2() {
        let r = JSONifyPayloadRegistry::v2();
        for (name, version) in [
            ("__Versioned_rpc.Menu", 1),
            ("get_some_initial_peers", 1),
            ("get_staged_ledger_aux_and_pending_coinbases_at_hash", 2),
            ("answer_sync_ledger_query", 2),
            ("get_transition_chain", 2),
            ("get_transition_chain_proof", 1),
            ("Get_transition_knowledge", 1),
            ("get_ancestry", 2),
            ("ban_notify", 1),
            ("get_best_tip", 2),
            ("get_node_status", 1),
            ("get_node_status", 2),
            ("get_epoch_ledger", 2),
        ] {
            assert!(r.get(name, version).is_some());
        }
    }

    #[test]
    fn jsonify_registry_query() {
        let r = JSONifyPayloadRegistry::v1();
        let payload =
            hex::decode("220101e7dd9b0d45abb2e4dec2c5d22e1f1bd8ae5133047914209a0229e90a62ecfb0e")
                .unwrap();
        let mut ptr = payload.as_slice();
        let jsonify = r.get("get_transition_chain", 1).unwrap();
        let json = jsonify.read_query(&mut ptr).unwrap();
        let expected = serde_json::json!([
            "0x0efbec620ae929029a201479043351aed81b1f2ed2c5c2dee4b2ab450d9bdde7"
        ]);
        assert_eq!(json, expected);
    }
}

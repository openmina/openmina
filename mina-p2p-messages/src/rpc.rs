///! Mina RPC methods
use std::collections::BTreeMap;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::core;
use crate::rpc_kernel::*;
use crate::{
    core::InetAddrV1Binable,
    string::CharString,
    v2::MinaBaseSparseLedgerBaseStableV2,
    versioned::{Ver, Versioned},
};
use crate::{v1, v2};

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
    Vec<v1::NetworkPeerPeerIdStableV1Binable>
);

pub type GetStagedLedgerAuxAndPendingCoinbasesAtHashV1Response = Option<(
    v1::TransactionSnarkScanStateStableV1Binable,
    LedgerHashV1Binable,
    v1::MinaBasePendingCoinbaseStableV1Binable,
    Vec<v1::MinaStateProtocolStateValueStableV1Binable>,
)>;

mina_rpc!(
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV1,
    "get_staged_ledger_aux_and_pending_coinbases_at_hash",
    1,
    StateHashV1Binable,
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
    (LedgerHashV1Binable, v1::MinaBaseSyncLedgerQueryStableV1Binable),
    RpcResult<v1::MinaBaseSyncLedgerAnswerStableV1Binable, core::Error>
);

mina_rpc!(
    AnswerSyncLedgerQueryV2,
    "answer_sync_ledger_query",
    2,
    (LedgerHashV1, v2::MinaLedgerSyncLedgerQueryStableV1),
    RpcResult<v2::MinaLedgerSyncLedgerAnswerStableV2, core::Error>
);

mina_rpc!(
    GetTransitionChainV1,
    "get_transition_chain",
    1,
    Vec<StateHashV1Binable>,
    Option<Vec<v1::MinaBlockExternalTransitionRawVersionedStableV1Binable>>
);

mina_rpc!(
    GetTransitionChainV2,
    "get_transition_chain",
    2,
    Vec<StateHashV1>,
    Option<Vec<v2::MinaBlockBlockStableV2>>
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
pub struct ProofCarryingDataWithHashV1<A, B> {
    data: A,
    proof: B,
}
pub type ProofCarryingDataWithHashV1Binable<A, B> = Versioned<ProofCarryingDataWithHashV1<A, B>, 1>;

pub type GetAncestryV1Query = WithHashV1Binable<
    v1::ConsensusProofOfStakeDataConsensusStateValueStableV1Binable,
    StateHashV1Binable,
>;
pub type GetAncestryV1Response = Option<
    ProofCarryingDataWithHashV1Binable<
        v1::MinaBlockExternalTransitionRawVersionedStableV1Binable,
        (
            Vec<StateBodyHashV1Binable>,
            v1::MinaBlockExternalTransitionRawVersionedStableV1Binable,
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
    WithHashV1<v2::ConsensusProofOfStakeDataConsensusStateValueStableV1, StateHashV1>;
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
    data: A,
    proof: B,
}
pub type ProofCarryingDataStableV1Binable<A, B> = Versioned<ProofCarryingDataStableV1<A, B>, 1>;
pub type GetBestTipV1Response = Option<
    ProofCarryingDataStableV1Binable<
        v1::MinaBlockExternalTransitionRawVersionedStableV1Binable,
        (
            Vec<LedgerHashV1Binable>,
            v1::MinaBlockExternalTransitionRawVersionedStableV1Binable,
        ),
    >,
>;
pub type GetBestTipV2Response = Option<
    ProofCarryingDataStableV1<
        v2::MinaBlockBlockStableV2,
        (Vec<LedgerHashV1>, v2::MinaBlockBlockStableV2),
    >,
>;
mina_rpc!(GetBestTipV1, "get_best_tip", 1, (), GetBestTipV1Response);
mina_rpc!(GetBestTipV2, "get_best_tip", 2, (), GetBestTipV2Response);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NodeStatusV1 {
    node_ip_addr: InetAddrV1Binable,
    node_peer_id: v1::NetworkPeerPeerIdStableV1Binable,
    sync_status: v1::SyncStatusTStableV1Binable,
    peers: Vec<v1::NetworkPeerPeerIdStableV1Binable>,
    block_producers: Vec<v1::PublicKeyCompressedStableV1Binable>,
    ban_statuses: Vec<(
        v1::NetworkPeerPeerIdStableV1Binable,
        v1::TrustSystemPeerStatusStableV1Binable,
    )>,
    k_block_hashes_and_timestamps: Vec<(StateHashV1Binable, CharString)>,
    git_commit: CharString,
    uptime_minutes: i32,
}
mina_rpc!(GetNodeStatusV1, "get_node_status", 1, (), RpcResult<NodeStatusV1, core::Error>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NodeStatusV2 {
    node_ip_addr: InetAddrV1Binable,
    node_peer_id: v1::NetworkPeerPeerIdStableV1Binable,
    sync_status: v1::SyncStatusTStableV1Binable,
    peers: Vec<v1::NetworkPeerPeerIdStableV1Binable>,
    block_producers: Vec<v1::PublicKeyCompressedStableV1Binable>,
    protocol_state_hash: StateHashV1Binable,
    ban_statuses: Vec<(
        v1::NetworkPeerPeerIdStableV1Binable,
        v1::TrustSystemPeerStatusStableV1Binable,
    )>,
    k_block_hashes_and_timestamps: Vec<(StateHashV1Binable, CharString)>,
    git_commit: CharString,
    uptime_minutes: i32,
    block_height_opt: Option<i32>,
}
mina_rpc!(GetNodeStatusV2, "get_node_status", 2, (), RpcResult<NodeStatusV2, core::Error>);

mina_rpc!(GetEpochLedgerV1, "get_epoch_ledger", 1, LedgerHashV1Binable, RpcResult<v1::MinaBaseSparseLedgerStableV1Binable, CharString>);

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

//! Mina RPC methods
use std::collections::BTreeMap;

use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::common::*;
use crate::core;
use crate::list::List;
use crate::rpc_kernel::*;
use crate::v2;
use crate::{
    core::InetAddrV1Versioned,
    string::CharString,
    v2::MinaBaseSparseLedgerBaseStableV2,
    versioned::{Ver, Versioned},
};

macro_rules! mina_rpc {
    ($name:ident, $tag:literal, $version:literal, $query:ty, $response:ty $(,)?) => {
        #[derive(Debug)]
        pub struct $name;
        impl crate::rpc_kernel::RpcMethod for $name {
            const NAME: crate::rpc_kernel::RpcTag = $tag.as_bytes();
            const NAME_STR: &'static str = $tag;
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
    GetSomeInitialPeersV1ForV2,
    "get_some_initial_peers",
    1,
    (),
    Vec<v2::NetworkPeerPeerStableV1>
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
    AnswerSyncLedgerQueryV2,
    "answer_sync_ledger_query",
    3,
    (LedgerHashV1, v2::MinaLedgerSyncLedgerQueryStableV1),
    RpcResult<v2::MinaLedgerSyncLedgerAnswerStableV2, core::Error>
);

mina_rpc!(
    GetTransitionChainV2,
    "get_transition_chain",
    2,
    Vec<StateHashV1>,
    Option<Vec<v2::MinaBlockBlockStableV2>>
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
pub type GetBestTipV2Response = Option<
    ProofCarryingDataStableV1<
        v2::MinaBlockBlockStableV2,
        (
            Vec<v2::MinaBaseStateBodyHashStableV1>,
            v2::MinaBlockBlockStableV2,
        ),
    >,
>;
mina_rpc!(GetBestTipV2, "get_best_tip", 2, (), GetBestTipV2Response);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NodeStatusV2 {
    node_ip_addr: InetAddrV1Versioned,
    node_peer_id: v2::NetworkPeerPeerIdStableV1,
    sync_status: v2::SyncStatusTStableV1,
    peers: List<v2::NetworkPeerPeerIdStableV1>,
    block_producers: List<v2::NonZeroCurvePoint>,
    protocol_state_hash: v2::StateHash,
    ban_statuses: List<(
        v2::NetworkPeerPeerIdStableV1,
        v2::TrustSystemPeerStatusStableV1,
    )>,
    k_block_hashes_and_timestamps: List<(v2::StateHash, CharString)>,
    git_commit: CharString,
    uptime_minutes: i32,
    block_height_opt: Option<i32>,
}
mina_rpc!(GetNodeStatusV2, "get_node_status", 2, (), RpcResult<NodeStatusV2, core::Error>);

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
    table: BTreeMap<(&'static [u8], Ver), Box<dyn JSONinifyPayloadReader>>,
}

impl Default for JSONifyPayloadRegistry {
    fn default() -> Self {
        Self::v2()
    }
}

impl JSONifyPayloadRegistry {
    #[deprecated = "Use `[v2]` method instead."]
    pub fn new() -> Self {
        Self::v2()
    }

    /// Creates registry with Mina V2 specific RPC methods.
    pub fn v2() -> Self {
        let mut this = Self {
            table: BTreeMap::new(),
        };
        this.insert(GetSomeInitialPeersV1ForV2);
        this.insert(GetStagedLedgerAuxAndPendingCoinbasesAtHashV2);
        this.insert(AnswerSyncLedgerQueryV2);
        this.insert(GetTransitionChainV2);
        this.insert(GetTransitionChainProofV1ForV2);
        this.insert(GetTransitionKnowledgeV1ForV2);
        this.insert(BanNotifyV1);
        this.insert(GetAncestryV2);
        this.insert(GetBestTipV2);
        this.insert(GetNodeStatusV2);
        this.insert(GetEpochLedgerV2);
        this
    }

    pub fn get<'a, 'b: 'a>(
        &'a self,
        name: &'b [u8],
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
            ("get_node_status", 2),
            ("get_epoch_ledger", 2),
        ] {
            assert!(r.get(name.as_bytes(), version).is_some());
        }
    }

    #[test]
    fn jsonify_registry_query() {
        let r = JSONifyPayloadRegistry::v2();
        let payload =
            hex::decode("220101e7dd9b0d45abb2e4dec2c5d22e1f1bd8ae5133047914209a0229e90a62ecfb0e")
                .unwrap();
        let mut ptr = payload.as_slice();
        let jsonify = r.get(b"get_transition_chain", 1).unwrap();
        let json = jsonify.read_query(&mut ptr).unwrap();
        let expected = serde_json::json!([
            "0x0efbec620ae929029a201479043351aed81b1f2ed2c5c2dee4b2ab450d9bdde7"
        ]);
        assert_eq!(json, expected);
    }
}

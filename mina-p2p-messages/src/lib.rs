use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use rpc::RpcResult;
use serde::{Deserialize, Serialize};
use v1::{
    ConsensusProofOfStakeDataConsensusStateValueStableV1Binable,
    //TransactionSnarkScanStateStableV1Binable,
    MinaBasePendingCoinbaseStableV1Binable,
    MinaBaseSparseLedgerStableV1Binable,
    MinaBaseSyncLedgerAnswerStableV1Binable,
    MinaBaseSyncLedgerQueryStableV1Binable,
    MinaBlockExternalTransitionRawVersionedStableV1Binable,
    MinaStateProtocolStateValueStableV1Binable,
    NetworkPeerPeerIdStableV1Binable,
};
use versioned::Versioned;

pub mod bigint;
pub mod char_;
pub mod core_error;
pub mod phantom;
pub mod rpc;
pub mod string;
pub mod utils;
pub mod v1;
pub mod versioned;

pub type StateHashV1Binable = Versioned<bigint::BigInt, 1>;
pub type StateBodyHashV1Binable = Versioned<bigint::BigInt, 1>;
pub type LedgerHashV1Binable = Versioned<bigint::BigInt, 1>;

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[serde(tag = "type", content = "message")]
pub enum GossipNetMessage {
    #[serde(rename = "external_transition")]
    NewState(v1::MinaBlockExternalTransitionRawVersionedStableV1Binable),
    #[serde(rename = "snark_pool_diff")]
    SnarkPoolDiff(v1::NetworkPoolSnarkPoolDiffVersionedStableV1Binable),
    #[serde(rename = "transaction_pool_diff")]
    TransactionPoolDiff(v1::NetworkPoolTransactionPoolDiffVersionedStableV1Binable),
}

macro_rules! mina_rpc {
    ($name:ident, $tag:literal, $version:literal, $query:ty, $response:ty) => {
        pub struct $name;
        impl crate::rpc::RpcMethod for $name {
            const NAME: &'static str = $tag;
            const VERSION: crate::versioned::Ver = $version;
            type Query = $query;
            type Response = $response;
        }
    };
}

mina_rpc!(GetEpochLedger, "get_epoch_ledger", 1, LedgerHashV1Binable, RpcResult<MinaBaseSparseLedgerStableV1Binable, string::String>);

mina_rpc!(
    GetSomeInitialPeersV1,
    "get_some_initial_peers",
    1,
    (),
    Vec<NetworkPeerPeerIdStableV1Binable>
);

// TODO implement TransactionSnarkScanStateStableV1Binable
mina_rpc!(
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV1,
    "get_staged_ledger_aux_and_pending_coinbases_at_hash",
    1,
    StateHashV1Binable,
    Option<(
        (), //TransactionSnarkScanStateStableV1Binable,
        LedgerHashV1Binable,
        MinaBasePendingCoinbaseStableV1Binable,
        Vec<MinaStateProtocolStateValueStableV1Binable>
    )>
);

mina_rpc!(
    AnswerSyncLedgerQueryV1,
    "answer_sync_ledger_query",
    1,
    (LedgerHashV1Binable, MinaBaseSyncLedgerQueryStableV1Binable),
    RpcResult<MinaBaseSyncLedgerAnswerStableV1Binable, core_error::Error>
);

mina_rpc!(
    GetTransitionChainV1,
    "get_transition_chain",
    1,
    Vec<StateHashV1Binable>,
    Option<Vec<MinaBlockExternalTransitionRawVersionedStableV1Binable>>
);

mina_rpc!(
    GetTransitionChainProofV1,
    "get_transition_chain_proof",
    1,
    StateHashV1Binable,
    Option<(StateHashV1Binable, Vec<StateBodyHashV1Binable>)>
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

mina_rpc!(
    GetAncestryV1,
    "get_ancestry",
    1,
    WithHashV1Binable<ConsensusProofOfStakeDataConsensusStateValueStableV1Binable, StateHashV1Binable>,
    Option<
        ProofCarryingDataWithHashV1Binable<
            MinaBlockExternalTransitionRawVersionedStableV1Binable,
            (Vec<StateBodyHashV1Binable>, MinaBlockExternalTransitionRawVersionedStableV1Binable)
        >
    >
);

// mina_rpc!(BanNotify, "ban_notify", SystemTime, ());

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct ProofCarryingDataStableV1<A, B> {
    data: A,
    proof: B,
}
pub type ProofCarryingDataStableV1Binable<A, B> = Versioned<ProofCarryingDataStableV1<A, B>, 1>;
mina_rpc!(
    GetBestTipV1,
    "get_best_tip",
    1,
    (),
    Option<
        ProofCarryingDataStableV1Binable<
            MinaBlockExternalTransitionRawVersionedStableV1Binable,
            (
                Vec<LedgerHashV1Binable>,
                MinaBlockExternalTransitionRawVersionedStableV1Binable
            ),
        >,
    >
);

// pub struct NodeStatusV1 {}
// mina_rpc!(GetNodeStatus, "get_node_status", 1, (), Result<NodeStatus, Box<dyn std::error::Error>>);
// pub struct NodeStatus {}
// mina_rpc!(GetNodeStatus, "get_node_status", 2, (), Result<NodeStatus, Box<dyn std::error::Error>>);

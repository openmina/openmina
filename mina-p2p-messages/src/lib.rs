use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use rpc::RpcResult;
use serde::{Deserialize, Serialize};
use v1::{
    MinaBaseSparseLedgerStableV1Binable, MinaBlockExternalTransitionRawVersionedStableV1Binable,
    NetworkPeerPeerIdStableV1Binable,
};
use versioned::Versioned;

pub mod bigint;
pub mod char_;
pub mod phantom;
pub mod rpc;
pub mod string;
pub mod utils;
pub mod v1;
pub mod versioned;

pub type LedgerHash = Versioned<bigint::BigInt, 1>;
pub type StateBodyHash = Versioned<bigint::BigInt, 1>;

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

mina_rpc!(GetEpochLedger, "get_epoch_ledger", 1, LedgerHash, RpcResult<MinaBaseSparseLedgerStableV1Binable, string::String>);

mina_rpc!(
    GetSomeInitialPeersV1,
    "get_some_initial_peers",
    1,
    (),
    Vec<NetworkPeerPeerIdStableV1Binable>
);

// pub struct StateHash;
// pub struct ScanState;
// pub struct LedgerHash;
// pub struct PendingCoinbase;
// pub struct ProtocolState;
// mina_rpc!(GetStagedLedgerAuxAndPendingCoinbasesAtHash, "get_staged_ledger_aux_and_pending_coinbases_at_hash", StateHash, Option<(ScanState, LedgerHash, PendingCoinbase, Vec<ProtocolState>)>);

// pub struct SyncLedgerQuery;
// pub struct SyncLedgerAnswer;
// mina_rpc!(AnswerSyncLedgerQuery, "answer_sync_ledger_query", (LedgerHash, SyncLedgerQuery),  Result<SyncLedgerAnswer, Box<dyn std::error::Error>>);

// pub struct MinaBlockStable;
// mina_rpc!(GetTransitionChain, "get_transition_chain", Vec<StateHash>, Option<Vec<MinaBlockStable>>);

// pub struct StateBodyHash;
// mina_rpc!(GetTransitionChainProof, "get_transition_chain_proof", StateHash, Option<(StateHash, Vec<StateBodyHash>)>);

// mina_rpc!(GetTransitionKnowledge, "Get_transition_knowledge", (), Vec<StateHash>);

// pub struct ConsensusDataConsensusStateValue;
// pub struct WithHash<T, U>(PhantomData<T>, PhantomData<U>);
// pub struct ProofCarryingData<T, U> {
//     __1: PhantomData<T>,
//     __2: PhantomData<U>,
// }
// mina_rpc!(GetAncestry, "get_ancestry", WithHash<ConsensusDataConsensusStateValue, StateHash>, Option<ProofCarryingData<MinaBlockStable, (Vec<StateBodyHash>, MinaBlockStable)>>);

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
                Vec<StateBodyHash>,
                MinaBlockExternalTransitionRawVersionedStableV1Binable
            ),
        >,
    >
);

// pub struct NodeStatusV1 {}
// mina_rpc!(GetNodeStatus, "get_node_status", 1, (), Result<NodeStatus, Box<dyn std::error::Error>>);
// pub struct NodeStatus {}
// mina_rpc!(GetNodeStatus, "get_node_status", 2, (), Result<NodeStatus, Box<dyn std::error::Error>>);

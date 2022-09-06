use binprot_derive::{BinProtRead, BinProtWrite};
use rpc::RpcResult;
use serde::{Deserialize, Serialize};
use v1::MinaBaseSparseLedgerStableV1Binable;
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
    ($name:ident, $tag:literal, $query:ty, $response:ty) => {
        pub struct $name;
        impl crate::rpc::RpcMethod for $name {
            const NAME: &'static str = $tag;
            type Query = $query;
            type Response = $response;
        }
    };
}

mina_rpc!(GetEpochLedger, "get_epoch_ledger", LedgerHash, RpcResult<MinaBaseSparseLedgerStableV1Binable, string::String>);

// pub struct NetworkPeer;
// mina_rpc!(GetSomeInitialPeers, "get_some_initial_peers", (), Vec<NetworkPeer>);

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

// mina_rpc!(GetBestTip, "get_best_tip", (), Option<ProofCarryingData<MinaBlockStable, (Vec<StateBodyHash>, MinaBlockStable)>>);

// pub struct NodeStatus {}
// mina_rpc!(GetNodeStatus, "get_node_status", (), Result<NodeStatus, Box<dyn std::error::Error>>);

mod ledger_read_actions;
use ledger::{Account, AccountId};
pub use ledger_read_actions::*;

mod ledger_read_state;
pub use ledger_read_state::*;
use mina_core::{
    block::AppliedBlock,
    requests::{RequestId, RpcId, RpcIdType},
};
use p2p::{channels::rpc::P2pRpcId, PeerId};
use redux::Callback;

mod ledger_read_reducer;

use std::{collections::BTreeMap, sync::Arc};

use mina_p2p_messages::v2;
use serde::{Deserialize, Serialize};

use crate::{
    account::AccountPublicKey,
    block_producer::vrf_evaluator::DelegatorTable,
    ledger::LedgerAddress,
    p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases,
    rpc::{AccountQuery, RpcScanStateSummaryScanStateJob},
};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum LedgerReadKind {
    DelegatorTable,
    GetNumAccounts,
    GetAccounts,
    GetChildHashesAtAddr,
    GetChildAccountsAtAddr,
    GetStagedLedgerAuxAndPendingCoinbases,
    ScanStateSummary,
    AccountsForRpc,
    GetLedgerStatus,
    GetAccountDelegators,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum LedgerReadRequest {
    /// Delegator table requested by vrf state machine.
    DelegatorTable(v2::LedgerHash, AccountPublicKey),
    // p2p rpcs
    GetNumAccounts(v2::LedgerHash),
    GetAccounts(v2::LedgerHash, Vec<AccountId>, Option<RpcId>),
    GetChildHashesAtAddr(v2::LedgerHash, LedgerAddress),
    GetChildAccountsAtAddr(v2::LedgerHash, LedgerAddress),
    GetStagedLedgerAuxAndPendingCoinbases(LedgerReadStagedLedgerAuxAndPendingCoinbases),
    // rpcs
    ScanStateSummary(v2::MinaBaseStagedLedgerHashStableV1),
    AccountsForRpc(RpcId, v2::LedgerHash, AccountQuery),
    GetLedgerStatus(RpcId, v2::LedgerHash),
    GetAccountDelegators(RpcId, v2::LedgerHash, AccountId),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LedgerStatus {
    pub num_accounts: u64,
    pub best_tip_staged_ledger_hash: v2::LedgerHash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerReadResponse {
    /// Delegator table requested by vrf state machine.
    DelegatorTable(Option<DelegatorTable>),
    // p2p rpcs
    GetNumAccounts(Option<(u64, v2::LedgerHash)>),
    GetAccounts(Vec<Account>, Option<RpcId>),
    GetChildHashesAtAddr(Option<(v2::LedgerHash, v2::LedgerHash)>),
    GetChildAccountsAtAddr(Option<Vec<v2::MinaBaseAccountBinableArgStableV2>>),
    GetStagedLedgerAuxAndPendingCoinbases(Option<Arc<StagedLedgerAuxAndPendingCoinbases>>),
    // rpcs
    ScanStateSummary(Result<Vec<Vec<RpcScanStateSummaryScanStateJob>>, String>),
    AccountsForRpc(RpcId, Vec<Account>, AccountQuery),
    GetLedgerStatus(RpcId, Option<LedgerStatus>),
    GetAccountDelegators(RpcId, Option<Vec<Account>>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerReadStagedLedgerAuxAndPendingCoinbases {
    pub ledger_hash: v2::MinaBaseStagedLedgerHashStableV1,
    pub protocol_states: BTreeMap<v2::StateHash, v2::MinaStateProtocolStateValueStableV2>,
}

impl LedgerReadRequest {
    pub fn kind(&self) -> LedgerReadKind {
        match self {
            Self::DelegatorTable(..) => LedgerReadKind::DelegatorTable,
            Self::GetNumAccounts(..) => LedgerReadKind::GetNumAccounts,
            Self::GetAccounts(..) => LedgerReadKind::GetAccounts,
            Self::GetChildAccountsAtAddr(..) => LedgerReadKind::GetChildAccountsAtAddr,
            Self::GetChildHashesAtAddr(..) => LedgerReadKind::GetChildHashesAtAddr,
            Self::GetStagedLedgerAuxAndPendingCoinbases(..) => {
                LedgerReadKind::GetStagedLedgerAuxAndPendingCoinbases
            }
            Self::ScanStateSummary(..) => LedgerReadKind::ScanStateSummary,
            Self::AccountsForRpc(..) => LedgerReadKind::AccountsForRpc,
            Self::GetLedgerStatus(..) => LedgerReadKind::GetLedgerStatus,
            Self::GetAccountDelegators(..) => LedgerReadKind::GetAccountDelegators,
        }
    }

    pub fn cost(&self) -> usize {
        let cost = match self {
            Self::DelegatorTable(..) => 100,
            Self::GetNumAccounts(..) => 1,
            Self::GetAccounts(..) => 10, // Not sure if 10 is a good number here
            Self::GetChildAccountsAtAddr(_, addr) => {
                let height_diff = super::LEDGER_DEPTH.saturating_sub(addr.length());
                let max_accounts_count = 2_u32.pow(height_diff as u32);
                (max_accounts_count / 4) as usize
            }
            Self::GetChildHashesAtAddr(..) => 1,
            Self::GetStagedLedgerAuxAndPendingCoinbases(..) => 100,
            Self::ScanStateSummary(..) => 100,
            // TODO(adonagy): not sure
            Self::AccountsForRpc(..) => 10,
            Self::GetLedgerStatus(..) => 1,
            Self::GetAccountDelegators(..) => 10,
        };
        cost.max(1)
    }
}

impl LedgerReadResponse {
    pub fn kind(&self) -> LedgerReadKind {
        match self {
            Self::DelegatorTable(..) => LedgerReadKind::DelegatorTable,
            Self::GetNumAccounts(..) => LedgerReadKind::GetNumAccounts,
            Self::GetAccounts(..) => LedgerReadKind::GetAccounts,
            Self::GetChildAccountsAtAddr(..) => LedgerReadKind::GetChildAccountsAtAddr,
            Self::GetChildHashesAtAddr(..) => LedgerReadKind::GetChildHashesAtAddr,
            Self::GetStagedLedgerAuxAndPendingCoinbases(..) => {
                LedgerReadKind::GetStagedLedgerAuxAndPendingCoinbases
            }
            Self::ScanStateSummary(..) => LedgerReadKind::ScanStateSummary,
            Self::AccountsForRpc(..) => LedgerReadKind::AccountsForRpc,
            Self::GetLedgerStatus(..) => LedgerReadKind::GetLedgerStatus,
            Self::GetAccountDelegators(..) => LedgerReadKind::GetAccountDelegators,
        }
    }
}

impl PartialEq for LedgerReadStagedLedgerAuxAndPendingCoinbases {
    fn eq(&self, other: &Self) -> bool {
        self.ledger_hash == other.ledger_hash
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerReadInitCallback {
    RpcLedgerAccountsGetPending {
        callback: Callback<RequestId<RpcIdType>>,
        args: RequestId<RpcIdType>,
    },
    RpcScanStateSummaryGetPending {
        callback: Callback<(RequestId<RpcIdType>, AppliedBlock)>,
        args: (RequestId<RpcIdType>, AppliedBlock),
    },
    P2pChannelsResponsePending {
        callback: Callback<(bool, P2pRpcId, PeerId)>,
        args: (bool, P2pRpcId, PeerId),
    },
    RpcLedgerStatusGetPending {
        callback: Callback<RequestId<RpcIdType>>,
        args: RequestId<RpcIdType>,
    },
    RpcLedgerAccountDelegatorsGetPending {
        callback: Callback<RequestId<RpcIdType>>,
        args: RequestId<RpcIdType>,
    },
    None,
}

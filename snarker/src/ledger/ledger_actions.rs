use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};
use serde::{Deserialize, Serialize};

use super::{LedgerAddress, LedgerId};

pub type LedgerActionWithMeta = redux::ActionWithMeta<LedgerAction>;
pub type LedgerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a LedgerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum LedgerAction {
    ChildHashesAdd(LedgerChildHashesAddAction),
    ChildAccountsAdd(LedgerChildAccountsAddAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerChildHashesAddAction {
    pub ledger_id: LedgerId,
    pub parent: LedgerAddress,
    pub hashes: (LedgerHash, LedgerHash),
}

impl redux::EnablingCondition<crate::State> for LedgerChildHashesAddAction {
    fn is_enabled(&self, _state: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerChildAccountsAddAction {
    pub ledger_id: LedgerId,
    pub parent: LedgerAddress,
    pub accounts: Vec<MinaBaseAccountBinableArgStableV2>,
}

impl redux::EnablingCondition<crate::State> for LedgerChildAccountsAddAction {
    fn is_enabled(&self, _state: &crate::State) -> bool {
        true
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::Ledger(value.into())
            }
        }
    };
}

impl_into_global_action!(LedgerChildHashesAddAction);
impl_into_global_action!(LedgerChildAccountsAddAction);

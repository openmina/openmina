/// <https://github.com/MinaProtocol/mina/blob/1551e2faaa246c01636908aabe5f7981715a10f4/src/lib/mina_base/zkapp_command.ml#L1421>
use super::{AccountUpdate, CallForest, FeePayer, Memo};

pub fn account_update(_: &AccountUpdate) -> u64 {
    1
}

pub fn fee_payer(_: &FeePayer) -> u64 {
    1
}

pub fn account_updates(list: &CallForest<AccountUpdate>) -> u64 {
    list.fold(0, |acc, p| acc + account_update(p))
}

pub fn memo(_: &Memo) -> u64 {
    0
}

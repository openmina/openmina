use redux::ActionWithMeta;

use crate::{
    ledger::{
        read::{LedgerReadAction, LedgerReadInitCallback},
        write::LedgerWriteAction,
        LedgerService,
    },
    Store,
};

use super::LedgerEffectfulAction;

pub fn ledger_effectful_effects<S>(
    store: &mut Store<S>,
    action: ActionWithMeta<LedgerEffectfulAction>,
) where
    S: LedgerService,
{
    let (action, _meta) = action.split();

    match action {
        LedgerEffectfulAction::WriteInit { request, on_init } => {
            store.service.write_init(request.clone());
            store.dispatch(LedgerWriteAction::Pending);
            store.dispatch_callback(on_init, request);
        }
        LedgerEffectfulAction::ReadInit {
            request,
            callback,
            id,
        } => {
            store.service.read_init(id, request.clone());
            store.dispatch(LedgerReadAction::Pending { id, request });

            match callback {
                LedgerReadInitCallback::RpcLedgerAccountsGetPending { callback, args } => {
                    store.dispatch_callback(callback, args);
                }
                LedgerReadInitCallback::RpcScanStateSummaryGetPending { callback, args } => {
                    store.dispatch_callback(callback, args);
                }
                LedgerReadInitCallback::P2pChannelsResponsePending { callback, args } => {
                    store.dispatch_callback(callback, args);
                }
                LedgerReadInitCallback::None => {}
            }
        }
    }
}

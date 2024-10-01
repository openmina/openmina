use openmina_core::fuzzed_maybe;

use super::P2pNetworkPnetEffectfulAction;
use crate::P2pMioService;
use crate::{P2pNetworkSelectAction, SelectKind};

impl P2pNetworkPnetEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
    {
        let service = store.service();

        match self {
            P2pNetworkPnetEffectfulAction::OutgoingData { addr, data } => {
                let data = fuzzed_maybe!(data, crate::fuzzer::mutate_pnet);
                service.send_mio_cmd(crate::MioCmd::Send(addr, data.into_boxed_slice()));
            }
            P2pNetworkPnetEffectfulAction::SetupNonce {
                addr,
                nonce,
                incoming,
            } => {
                service.send_mio_cmd(crate::MioCmd::Send(addr, nonce.to_vec().into_boxed_slice()));
                store.dispatch(P2pNetworkSelectAction::Init {
                    addr,
                    kind: SelectKind::Authentication,
                    incoming,
                });
            }
        }
    }
}

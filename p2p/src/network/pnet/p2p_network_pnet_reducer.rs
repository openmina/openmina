use openmina_core::Substate;
use salsa_simple::XSalsa20;

use crate::{
    disconnection::P2pDisconnectionReason, Data, P2pNetworkPnetEffectfulAction,
    P2pNetworkSchedulerAction, P2pNetworkSchedulerState, P2pNetworkSelectAction,
};

use super::{
    p2p_network_pnet_state::{Half, P2pNetworkPnetState},
    *,
};

impl Half {
    fn reduce(&mut self, shared_secret: &[u8; 32], data: &[u8]) {
        match self {
            Half::Buffering { buffer, offset } => {
                if *offset + data.len() < 24 {
                    buffer[*offset..(*offset + data.len())].clone_from_slice(data);
                    *offset += data.len();
                } else {
                    if *offset < 24 {
                        buffer[*offset..24].clone_from_slice(&data[..(24 - *offset)]);
                    }
                    let nonce = *buffer;
                    let remaining = data[(24 - *offset)..].to_vec().into_boxed_slice();
                    *self = Half::Done {
                        cipher: XSalsa20::new(*shared_secret, nonce),
                        to_send: vec![],
                    };
                    self.reduce(shared_secret, &remaining);
                }
            }
            Half::Done { cipher, to_send } => {
                *to_send = data.to_vec();
                cipher.apply_keystream(to_send.as_mut());
            }
        }
    }
}

impl P2pNetworkPnetState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pNetworkSchedulerState>,
        action: redux::ActionWithMeta<&P2pNetworkPnetAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, _meta) = action.split();
        let pnet_state = &mut state_context
            .get_substate_mut()?
            .connection_state_mut(action.addr())
            .ok_or_else(|| format!("Missing connection for action: {:?}", action))?
            .pnet;

        match action {
            P2pNetworkPnetAction::IncomingData { data, addr } => {
                pnet_state.incoming.reduce(&pnet_state.shared_secret, data);

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let scheduler_state: &P2pNetworkSchedulerState = state.substate()?;
                let pnet_state = &scheduler_state
                    .connection_state(addr)
                    .ok_or_else(|| format!("Missing connection for action: {:?}", action))?
                    .pnet;

                let Half::Done { to_send, .. } = &pnet_state.incoming else {
                    return Ok(());
                };

                if !to_send.is_empty() {
                    let data = Data::from(to_send.clone());
                    dispatcher.push(P2pNetworkSelectAction::IncomingDataAuth {
                        addr: *addr,
                        data,
                        fin: false,
                    });
                }

                Ok(())
            }
            P2pNetworkPnetAction::OutgoingData { data, addr } => {
                pnet_state.outgoing.reduce(&pnet_state.shared_secret, data);

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let scheduler_state: &P2pNetworkSchedulerState = state.substate()?;
                let pnet_state = &scheduler_state
                    .connection_state(addr)
                    .ok_or_else(|| format!("Missing connection for action: {:?}", action))?
                    .pnet;

                let Half::Done { to_send, .. } = &pnet_state.outgoing else {
                    return Ok(());
                };

                if !to_send.is_empty() {
                    dispatcher.push(P2pNetworkPnetEffectfulAction::OutgoingData {
                        addr: *addr,
                        data: to_send.clone(),
                    });
                }
                Ok(())
            }
            P2pNetworkPnetAction::SetupNonce {
                nonce,
                addr,
                incoming,
            } => {
                pnet_state.outgoing.reduce(&pnet_state.shared_secret, nonce);

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkPnetEffectfulAction::SetupNonce {
                    addr: *addr,
                    nonce: nonce.clone(),
                    incoming: *incoming,
                });
                Ok(())
            }
            P2pNetworkPnetAction::Timeout { addr } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkSchedulerAction::Disconnect {
                    addr: *addr,
                    reason: P2pDisconnectionReason::Timeout,
                });
                Ok(())
            }
        }
    }
}

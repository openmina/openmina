use mina_p2p_messages::rpc::VersionedRpcMenuV1;
use serde::{Deserialize, Serialize};

use super::{outgoing::P2pRpcOutgoingState, P2pRpcKind};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcState {
    pub menu: Option<P2pRpcMenu>,
    pub outgoing: P2pRpcOutgoingState,
}

impl P2pRpcState {
    pub fn new() -> Self {
        Self {
            menu: None,
            outgoing: Default::default(),
        }
    }

    pub fn supports(&self, kind: P2pRpcKind) -> bool {
        match &self.menu {
            Some(menu) => menu.supports(kind),
            None => matches!(kind, P2pRpcKind::MenuGet),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcMenu {
    raw: <VersionedRpcMenuV1 as RpcMethod>::Response,
    map: [bool; 16],
}

use mina_p2p_messages::rpc_kernel::RpcMethod;
macro_rules! p2p_rpc_menu_map_set {
    ($raw: ident, $map: ident, $kind: ident, $rpc_method: ty) => {
        $map[P2pRpcKind::$kind as usize] = $raw.iter().any(|(name, ver)| {
            &name.to_string_lossy() == <$rpc_method as RpcMethod>::NAME
                && *ver == <$rpc_method as RpcMethod>::VERSION
        });
    };
}

impl P2pRpcMenu {
    pub fn new(raw: <VersionedRpcMenuV1 as RpcMethod>::Response) -> Self {
        let mut map = [false; 16];
        p2p_rpc_menu_map_set!(raw, map, MenuGet, VersionedRpcMenuV1);
        Self { raw, map }
    }

    pub fn supports(&self, kind: P2pRpcKind) -> bool {
        self.map[kind as usize]
    }
}

#[test]
fn p2p_rpc_menu_map_test() {
    // TODO(binier)
}

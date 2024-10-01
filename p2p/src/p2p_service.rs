pub use redux::TimeService;

pub use crate::channels::P2pChannelsService;
pub use crate::connection::P2pConnectionService;
pub use crate::disconnection_effectful::P2pDisconnectionService;

#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
pub use crate::{P2pCryptoService, P2pMioService, P2pNetworkService};

#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
pub trait P2pService:
    TimeService
    + P2pConnectionService
    + P2pDisconnectionService
    + P2pChannelsService
    + P2pMioService
    + P2pCryptoService
    + P2pNetworkService
{
}

#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
impl<T> P2pService for T where
    T: TimeService
        + P2pConnectionService
        + P2pDisconnectionService
        + P2pChannelsService
        + P2pMioService
        + P2pCryptoService
        + P2pNetworkService
{
}

#[cfg(not(all(not(target_arch = "wasm32"), feature = "p2p-libp2p")))]
pub trait P2pService:
    TimeService + P2pConnectionService + P2pDisconnectionService + P2pChannelsService
{
}

#[cfg(not(all(not(target_arch = "wasm32"), feature = "p2p-libp2p")))]
impl<T> P2pService for T where
    T: TimeService + P2pConnectionService + P2pDisconnectionService + P2pChannelsService
{
}

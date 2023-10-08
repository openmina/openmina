#[cfg(not(target_arch = "wasm32"))]
pub mod libp2p;
pub mod webrtc;
#[cfg(not(target_arch = "wasm32"))]
pub mod webrtc_with_libp2p;

use std::future::Future;

pub trait TaskSpawner: Send + Clone {
    fn spawn_main<F>(&self, name: &str, fut: F)
    where
        F: 'static + Send + Future;
}

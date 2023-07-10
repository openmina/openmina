// #[cfg(feature = "service_impl_webrtc_rs")]
pub mod libp2p;
pub mod webrtc_rs;
pub mod webrtc_rs_with_libp2p;

use std::future::Future;

pub trait TaskSpawner: Send + Clone {
    fn spawn_main<F>(&self, name: &str, fut: F)
    where
        F: 'static + Send + Future;
}

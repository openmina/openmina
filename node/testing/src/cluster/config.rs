use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ClusterConfig {
    port_range: Option<(u16, u16)>,
    all_rust_to_rust_use_webrtc: bool,
    use_debugger: bool,
}

impl ClusterConfig {
    pub fn new(use_debugger: bool) -> Self {
        ClusterConfig {
            use_debugger,
            ..Default::default()
        }
    }

    pub fn use_debugger(mut self) -> Self {
        self.use_debugger = true;
        self
    }

    pub fn port_range(&self) -> std::ops::RangeInclusive<u16> {
        let range = self.port_range.unwrap_or((11_000, 49_151));
        (range.0)..=(range.1)
    }

    pub fn set_all_rust_to_rust_use_webrtc(mut self) -> Self {
        assert!(cfg!(feature = "p2p-webrtc"));
        self.all_rust_to_rust_use_webrtc = true;
        self
    }

    pub fn all_rust_to_rust_use_webrtc(&self) -> bool {
        self.all_rust_to_rust_use_webrtc
    }

    pub fn is_use_debugger(&self) -> bool {
        self.use_debugger
    }
}

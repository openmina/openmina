#[cfg(not(target_family = "wasm"))]
pub use non_wasm::Instant;

#[cfg(target_family = "wasm")]
pub use wasm::Instant;

mod wasm {
    pub struct Instant;

    impl Instant {
        pub fn now() -> Self {
            Self
        }

        pub fn elapsed(&self) -> std::time::Duration {
            std::time::Duration::new(0, 0)
        }
    }
}

mod non_wasm {
    pub use std::time::Instant;
}

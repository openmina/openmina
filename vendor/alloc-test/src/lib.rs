// #[cfg(feature = "benchmark")]
// mod benchmark;

// #[cfg(feature = "benchmark")]
// pub use benchmark::*;

macro_rules! log {
    ($($tt:tt)*) => {
        {
            #[cfg(not(target_family = "wasm"))]
            println!($($tt)*);
            #[cfg(target_family = "wasm")]
            wasm_bindgen_test::console_log!($($tt)*);
        }
    };
}

pub mod alloc;
pub mod perf;
pub mod threshold;

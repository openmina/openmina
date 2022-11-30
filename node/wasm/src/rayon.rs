use js_sys::Promise;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_rayon::init_thread_pool;

fn get_num_cpus() -> usize {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = navigator, js_name = hardwareConcurrency)]
        static HARDWARE_CONCURRENCY: usize;
    }

    std::cmp::max(*HARDWARE_CONCURRENCY, 1)
}

/// This method must be called to initialize rayon.
/// This is an async function, and the verification code must be called only after `init_rayon` returned.
/// This must not be called from the main thread.
pub async fn init_rayon() -> Result<(), JsValue> {
    let num_cpus = get_num_cpus();

    let thread_pool: Promise = init_thread_pool(num_cpus);

    &JsFuture::from(thread_pool).await?;

    Ok(())
}

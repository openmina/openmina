use wasm_bindgen::JsValue;

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

    wasm_thread::spawn(move || {
        rayon::ThreadPoolBuilder::new()
            .spawn_handler(|thread| {
                wasm_thread::spawn(move || thread.run());
                Ok(())
            })
            .num_threads(num_cpus.max(2) - 1)
            .build_global()
            .map_err(|e| format!("{:?}", e))
    })
    .join_async()
    .await
    .unwrap()?;

    Ok(())
}

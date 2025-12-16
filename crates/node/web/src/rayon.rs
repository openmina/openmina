use ::node::core::thread;
use wasm_bindgen::JsValue;

/// This method must be called to initialize rayon.
/// This is an async function, and the verification code must be called only after `init_rayon` returned.
/// This must not be called from the main thread.
pub async fn init_rayon() -> Result<(), JsValue> {
    let num_cpus = thread::available_parallelism()
        .map_err(|err| format!("failed to get available parallelism: {err}"))?
        .get();

    thread::spawn(move || {
        rayon::ThreadPoolBuilder::new()
            .spawn_handler(|thread| {
                thread::spawn(move || thread.run());
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

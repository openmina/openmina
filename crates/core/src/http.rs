#[cfg(target_family = "wasm")]
mod http {
    use crate::thread;
    use wasm_bindgen::prelude::*;

    fn to_io_err(err: JsValue) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{err:?}"))
    }

    async fn _get_bytes(url: String) -> std::io::Result<Vec<u8>> {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::Response;

        // let window = js_sys::global().dyn_into::<web_sys::WorkerGlobalScope>().unwrap();
        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_str(&url))
            .await
            .map_err(to_io_err)?;

        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();
        let js = JsFuture::from(resp.array_buffer().map_err(to_io_err)?)
            .await
            .map_err(to_io_err)?;
        Ok(js_sys::Uint8Array::new(&js).to_vec())
    }

    pub async fn get_bytes(url: &str) -> std::io::Result<Vec<u8>> {
        let url = url.to_owned();
        if thread::is_web_worker_thread() {
            thread::run_async_fn_in_main_thread(move || _get_bytes(url)).await.expect("failed to run task in the main thread! Maybe main thread crashed or not initialized?")
        } else {
            _get_bytes(url).await
        }
    }

    pub fn get_bytes_blocking(url: &str) -> std::io::Result<Vec<u8>> {
        let url = url.to_owned();
        if thread::is_web_worker_thread() {
            thread::run_async_fn_in_main_thread_blocking(move || _get_bytes(url)).expect("failed to run task in the main thread! Maybe main thread crashed or not initialized?")
        } else {
            panic!("can't do blocking requests from main browser thread");
        }
    }
}

#[cfg(target_family = "wasm")]
pub use http::{get_bytes, get_bytes_blocking};

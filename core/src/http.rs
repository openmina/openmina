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

    fn _download(filename: &str, data: &[u8]) -> std::io::Result<()> {
        let blob_input = js_sys::Array::of1(&js_sys::Uint8Array::from(data).into());
        let blob = web_sys::Blob::new_with_u8_slice_sequence(&blob_input).map_err(to_io_err)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(to_io_err)?;

        let document = web_sys::window()
            .and_then(|v| v.document())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "window.document object not available",
                )
            })?;
        let a = document
            .create_element("a")
            .map_err(to_io_err)?
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "window.document object not available",
                )
            })?;

        a.set_href(&url);
        a.set_download(filename);
        a.click();

        Ok(())
    }

    pub fn download(filename: String, data: Vec<u8>) -> std::io::Result<()> {
        if thread::is_web_worker_thread() {
            thread::run_async_fn_in_main_thread_blocking(move || async move { _download(&filename, &data) })
                .expect("failed to run task in the main thread! Maybe main thread crashed or not initialized?")
        } else {
            _download(&filename, &data)
        }
    }
}

#[cfg(target_family = "wasm")]
pub use http::{download, get_bytes, get_bytes_blocking};

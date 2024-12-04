use std::path::Path;

#[cfg(not(target_family = "wasm"))]
pub fn home_base_dir() -> Option<std::path::PathBuf> {
    let mut path = std::path::PathBuf::from(std::env::var("HOME").ok()?);
    path.push(".openmina/circuit-blobs");
    Some(path)
}

fn git_release_url(filename: &impl AsRef<Path>) -> String {
    const RELEASES_PATH: &str = "https://github.com/openmina/circuit-blobs/releases/download";
    let filename_str = filename.as_ref().to_str().unwrap();

    format!("{RELEASES_PATH}/{filename_str}")
}

#[cfg(not(target_family = "wasm"))]
pub fn fetch_blocking(filename: &impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    use std::path::PathBuf;

    fn try_base_dir<P: Into<PathBuf>>(base_dir: P, filename: &impl AsRef<Path>) -> Option<PathBuf> {
        let mut path = base_dir.into();
        path.push(filename);
        path.exists().then_some(path)
    }

    fn to_io_err(err: impl std::fmt::Display) -> std::io::Error {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "failed to find circuit-blobs locally and to fetch the from github! error: {err}"
            ),
        )
    }

    let home_base_dir = home_base_dir();
    let found = None
        .or_else(|| {
            try_base_dir(
                std::env::var("OPENMINA_CIRCUIT_BLOBS_BASE_DIR").ok()?,
                filename,
            )
        })
        .or_else(|| try_base_dir(env!("CARGO_MANIFEST_DIR").to_string(), filename))
        .or_else(|| try_base_dir(home_base_dir.clone()?, filename))
        .or_else(|| try_base_dir("/usr/local/lib/openmina/circuit-blobs", filename));

    if let Some(path) = found {
        return std::fs::read(path);
    }

    openmina_core::info!(
        openmina_core::log::system_time();
        kind = "ledger proofs",
        message = "circuit-blobs not found locally, so fetching it...",
        filename = filename.as_ref().to_str().unwrap(),
    );

    let base_dir = home_base_dir.expect("$HOME env not set!");

    let bytes = reqwest::blocking::get(git_release_url(filename))
        .map_err(to_io_err)?
        .bytes()
        .map_err(to_io_err)?
        .to_vec();

    // cache it to home dir.
    let cache_path = base_dir.join(filename);
    openmina_core::info!(
        openmina_core::log::system_time();
        kind = "ledger proofs",
        message = "caching circuit-blobs",
        path = cache_path.to_str().unwrap(),
    );
    let _ = std::fs::create_dir_all(cache_path.parent().unwrap());
    let _ = std::fs::write(cache_path, &bytes);

    Ok(bytes)
}

#[cfg(target_family = "wasm")]
pub async fn fetch(filename: &impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let prefix =
        option_env!("CIRCUIT_BLOBS_HTTP_PREFIX").unwrap_or("/assets/webnode/circuit-blobs");
    let url = format!("{prefix}/{}", filename.as_ref().to_str().unwrap());
    openmina_core::http::get_bytes(&url).await
    // http::get_bytes(&git_release_url(filename)).await
}

#[cfg(target_family = "wasm")]
pub fn fetch_blocking(filename: &impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let prefix =
        option_env!("CIRCUIT_BLOBS_HTTP_PREFIX").unwrap_or("/assets/webnode/circuit-blobs");
    let url = format!("{prefix}/{}", filename.as_ref().to_str().unwrap());
    openmina_core::http::get_bytes_blocking(&url)
}

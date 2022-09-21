#![allow(dead_code)]

use std::{path::{Path, PathBuf}, fs::File, io::Read};

pub fn files_path<P: AsRef<Path>>(suffix: P) -> std::io::Result<PathBuf> {
    let prefix = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "CARGO_MANIFEST_DIR variable")
    })?;
    Ok(PathBuf::from(prefix).join("tests/files").join(suffix))
}

pub fn read_file(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let _ = File::open(path)?.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn read(file: &str) -> std::io::Result<Vec<u8>> {
    read_file(&files_path(file)?)
}

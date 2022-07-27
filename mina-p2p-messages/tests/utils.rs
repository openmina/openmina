use std::{fs::File, io::Read, path::PathBuf};

pub fn read(file: &str) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let prefix = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "CARGO_MANIFEST_DIR variable")
    })?;
    let path = PathBuf::from(prefix).join("tests/files").join(file);
    eprintln!("{path:#?}");
    let _ = File::open(path)?.read_to_end(&mut buf)?;
    Ok(buf)
}

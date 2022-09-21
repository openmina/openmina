#![allow(dead_code)]

use std::{
    fmt::Debug,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use binprot::BinProtRead;

pub fn files_path<P: AsRef<Path>>(suffix: P) -> std::io::Result<PathBuf> {
    let prefix = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "CARGO_MANIFEST_DIR variable")
    })?;
    Ok(PathBuf::from(prefix).join("tests/files").join(suffix))
}

fn read_file(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let _ = File::open(path)?.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn read(file: &str) -> std::io::Result<Vec<u8>> {
    read_file(&files_path(file)?)
}

pub fn for_all<F>(dir: &str, mut f: F) -> std::io::Result<()>
where
    F: FnMut(&[u8]),
{
    let path = files_path(dir)?;
    let dir = std::fs::read_dir(path)?;
    for file in dir {
        let path = file?.path();
        if path.extension().map_or(false, |ext| ext == "bin") {
            println!("reading {path:?}...");
            f(&read_file(&path)?);
        }
    }
    Ok(())
}

pub fn for_all_with_path<F, A>(dir: A, mut f: F) -> std::io::Result<()>
where
    F: FnMut(&[u8], &Path),
    A: AsRef<Path>,
{
    let path = files_path(dir)?;
    let dir = std::fs::read_dir(path)?;
    for file in dir {
        let path = file?.path();
        if path.extension().map_or(false, |ext| ext == "bin") {
            println!("reading {path:?}...");
            f(&read_file(&path)?, &path);
        }
    }
    Ok(())
}

pub fn assert_binprot_read<T>(mut buf: &[u8])
where
    T: BinProtRead + Debug,
{
    let res = T::binprot_read(&mut buf);
    assert!(res.is_ok(), "{res:#?}");
    assert_eq!(buf.len(), 0);
}

pub fn assert_stream_read<T>(buf: &[u8])
where
    T: BinProtRead + Debug,
{
    use mina_p2p_messages::utils::FromBinProtStream;
    let mut p = buf;
    while !p.is_empty() {
        let res = T::read_from_stream(&mut p);
        assert!(res.is_ok(), "{res:#?}");
    }
}

pub fn assert_stream_read_and<T, F>(buf: &[u8], f: F)
where
    T: BinProtRead + Debug,
    F: Fn(Result<T, binprot::Error>),
{
    use mina_p2p_messages::utils::FromBinProtStream;
    let mut p = buf;
    while !p.is_empty() {
        f(T::read_from_stream(&mut p))
    }
}

pub fn stream_read_with<T, F>(buf: &[u8], mut f: F)
where
    T: BinProtRead + Debug,
    F: FnMut(Result<T, binprot::Error>, &[u8]),
{
    use mina_p2p_messages::utils::FromBinProtStream;
    let mut p = buf;
    while !p.is_empty() {
        let pp = p;
        let res = T::read_from_stream(&mut p);
        f(res, pp)
    }
}

#[macro_export]
macro_rules! binprot_read_test {
    (ignore($reason:literal), $name:ident, $path:expr, $ty:ty) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            utils::for_all($path, |encoded| utils::assert_binprot_read::<$ty>(&encoded)).unwrap();
        }
    };
    ($name:ident, $path:expr, $ty:ty) => {
        #[test]
        fn $name() {
            utils::for_all($path, |encoded| utils::assert_binprot_read::<$ty>(&encoded)).unwrap();
        }
    };
}

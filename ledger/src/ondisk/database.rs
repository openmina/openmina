use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use super::batch::Batch;

pub(super) type Key = Box<[u8]>;
pub(super) type Value = Box<[u8]>;
pub(super) type Offset = u64;

pub type Uuid = String;

pub struct Database {
    uuid: Uuid,
    index: HashMap<Key, Offset>,

    /// Points to end of file
    current_file_offset: Offset,
    file: BufWriter<std::fs::File>,

    buffer: Vec<u8>,

    filename: PathBuf,
}

impl Drop for Database {
    fn drop(&mut self) {
        eprintln!("\x1b[93mDatabase::drop {:?}\x1b[0m", self.filename);
    }
}

struct Header {
    key_length: u64,
    value_length: u64,
    is_removed: bool,
}

impl Header {
    pub const NBYTES: usize = 17;

    fn entry_length(&self) -> u64 {
        self.key_length + self.value_length
    }

    fn compute_value_offset(&self, header_offset: Offset) -> Offset {
        header_offset
            .checked_add(self.key_length)
            .unwrap()
            .checked_add(Header::NBYTES as u64)
            .unwrap()
    }

    fn make(key: &Key, value: &Option<Value>) -> std::io::Result<Self> {
        let to_u64 = |n: usize| {
            n.try_into()
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))
        };

        let is_removed = value.is_none();

        Ok(Header {
            key_length: to_u64(key.len())?,
            value_length: match value.as_ref() {
                None => 0,
                Some(value) => to_u64(value.len())?,
            },
            is_removed,
        })
    }

    fn read(bytes: &[u8]) -> std::io::Result<Self> {
        if bytes.len() < Self::NBYTES {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }

        let key_length = read_u64(bytes)?;
        let value_length = read_u64(&bytes[8..])?;
        let is_removed = read_bool(&bytes[16..])?;

        Ok(Self {
            key_length,
            value_length,
            is_removed,
        })
    }
}

pub fn next_uuid() -> Uuid {
    uuid::Uuid::new_v4().to_string()
}

fn read_u64(slice: &[u8]) -> std::io::Result<u64> {
    slice
        .get(..8)
        .and_then(|slice: &[u8]| slice.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(std::io::ErrorKind::UnexpectedEof.into())
}

fn read_bool(slice: &[u8]) -> std::io::Result<bool> {
    slice
        .get(..1)
        .and_then(|slice: &[u8]| slice.try_into().ok())
        .map(u8::from_le_bytes)
        .map(|b| b != 0)
        .ok_or(std::io::ErrorKind::UnexpectedEof.into())
}

fn ensure_buffer_length(buffer: &mut Vec<u8>, length: usize) {
    if buffer.len() < length {
        buffer.resize(length, 0)
    }
}

#[cfg(unix)]
fn read_exact_at(file: &mut File, buffer: &mut [u8], offset: Offset) -> std::io::Result<()> {
    use std::os::unix::prelude::FileExt;

    file.read_exact_at(buffer, offset)
}

#[cfg(not(unix))]
fn read_exact_at(file: &mut File, buffer: &mut [u8], offset: Offset) -> std::io::Result<()> {
    use std::io::Read;

    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(buffer)
}

enum CreateMode {
    Regular,
    Temporary,
}

impl Database {
    pub fn create(directory: impl AsRef<Path>) -> std::io::Result<Self> {
        Self::create_impl(directory, CreateMode::Regular)
    }

    fn create_impl(directory: impl AsRef<Path>, mode: CreateMode) -> std::io::Result<Self> {
        let directory = directory.as_ref();

        let filename = directory.join(match mode {
            CreateMode::Regular => "db",
            CreateMode::Temporary => "db_tmp",
        });

        if filename.try_exists()? {
            if let CreateMode::Temporary = mode {
                std::fs::remove_file(&filename)?;
            } else {
                eprintln!("\x1b[93mDatabase::reload {:?}\x1b[0m", directory);
                return Self::reload(filename);
            }
        }

        eprintln!("\x1b[93mDatabase::create {:?}\x1b[0m", directory);

        if !directory.try_exists()? {
            std::fs::create_dir_all(directory)?;
        }

        let file = std::fs::File::options()
            .read(true)
            .write(true)
            .append(true)
            .create_new(true)
            .open(&filename)?;

        Ok(Self {
            uuid: next_uuid(),
            index: HashMap::with_capacity(128),
            current_file_offset: 0,
            file: BufWriter::with_capacity(4 * 1024 * 1024, file), // 4 MB
            buffer: Vec::with_capacity(4096),
            filename,
        })
    }

    fn reload(filename: PathBuf) -> std::io::Result<Self> {
        use std::io::Read;

        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .append(true)
            .create_new(false)
            .open(&filename)?;

        let mut current_offset = 0;
        let eof = file.seek(SeekFrom::End(0))?;

        file.seek(SeekFrom::Start(0))?;

        let mut reader = BufReader::with_capacity(4096, file);
        let mut bytes = vec![0; 4096];

        let mut index = HashMap::with_capacity(256);

        while current_offset < eof {
            let header_offset = current_offset;

            ensure_buffer_length(&mut bytes, Header::NBYTES);
            reader.read_exact(&mut bytes[..Header::NBYTES])?;

            let header = Header::read(&bytes)?;
            let entry_length = header.entry_length() as usize;

            ensure_buffer_length(&mut bytes, entry_length);
            reader.read_exact(&mut bytes[..entry_length])?;

            let key_bytes = &bytes[..header.key_length as usize];

            if header.is_removed {
                index.remove(key_bytes);
            } else {
                index.insert(Box::<[u8]>::from(key_bytes), header_offset);
            }

            current_offset += (Header::NBYTES + entry_length) as u64;
        }

        if eof != current_offset {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }

        Ok(Self {
            uuid: next_uuid(),
            index,
            current_file_offset: eof,
            file: BufWriter::with_capacity(4 * 1024 * 1024, reader.into_inner()), // 4 MB
            buffer: Vec::with_capacity(4096),
            filename,
        })
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn close(&self) {
        eprintln!("\x1b[93mDatabase::close {:?}\x1b[0m", &self.filename);
        // TODO
    }

    fn read_header(&mut self, header_offset: Offset) -> std::io::Result<Header> {
        ensure_buffer_length(&mut self.buffer, Header::NBYTES);
        read_exact_at(
            self.file.get_mut(),
            &mut self.buffer[..Header::NBYTES],
            header_offset,
        )?;

        Header::read(&self.buffer)
    }

    fn read_value(&mut self, offset: Offset, length: usize) -> std::io::Result<&[u8]> {
        ensure_buffer_length(&mut self.buffer, length);
        read_exact_at(self.file.get_mut(), &mut self.buffer[..length], offset)?;

        Ok(&self.buffer[..length])
    }

    pub fn get_impl(&mut self, key: &[u8]) -> std::io::Result<Option<&[u8]>> {
        let header_offset = match self.index.get(key).copied() {
            Some(header_offset) => header_offset,
            None => return Ok(None),
        };

        let header = self.read_header(header_offset)?;

        let value_offset = header.compute_value_offset(header_offset);
        let value_length = header.value_length as usize;

        self.read_value(value_offset, value_length).map(Some)
    }

    /// `&mut self` is required for `File::seek`
    pub fn get(&mut self, key: &[u8]) -> std::io::Result<Option<Value>> {
        Ok(self.get_impl(key)?.map(Into::into))
    }

    fn set_impl(&mut self, key: Key, value: Option<Value>) -> std::io::Result<()> {
        let is_removed = value.is_none();
        let header = Header::make(&key, &value)?;

        let header_offset = self.current_file_offset;
        let is_removed_byte = if header.is_removed { 1 } else { 0 };

        self.file.write_all(&header.key_length.to_le_bytes())?;
        self.file.write_all(&header.value_length.to_le_bytes())?;
        self.file.write_all(&[is_removed_byte])?;
        self.file.write_all(&key)?;

        if let Some(value) = value.as_ref() {
            self.file.write_all(value)?;
        };

        let buffer_len = Header::NBYTES as u64 + header.entry_length();
        self.current_file_offset += buffer_len;

        // Update index
        if is_removed {
            self.index.remove(&key);
        } else {
            self.index.insert(key, header_offset);
        }

        Ok(())
    }

    pub fn set(&mut self, key: Key, value: Value) -> std::io::Result<()> {
        self.set_impl(key, Some(value))?;
        self.flush()?;
        Ok(())
    }

    pub fn set_batch(
        &mut self,
        key_data_pairs: impl IntoIterator<Item = (Key, Value)>,
        remove_keys: impl IntoIterator<Item = Key>,
    ) -> std::io::Result<()> {
        for (key, value) in key_data_pairs {
            self.set_impl(key, Some(value))?;
        }

        for key in remove_keys {
            self.set_impl(key, None)? // empty value
        }

        self.flush()?;

        Ok(())
    }

    pub fn get_batch(
        &mut self,
        keys: impl IntoIterator<Item = Key>,
    ) -> std::io::Result<Vec<Option<Value>>> {
        keys.into_iter().map(|key| self.get(&key)).collect()
    }

    pub fn make_checkpoint(&mut self, directory: impl AsRef<Path>) -> std::io::Result<()> {
        self.create_checkpoint(directory.as_ref())?;
        Ok(())
    }

    pub fn create_checkpoint(&mut self, directory: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut checkpoint = Self::create(directory.as_ref())?;

        let keys: Vec<Key> = self.index.keys().cloned().collect();

        for key in keys {
            let value = self.get(&key)?;
            checkpoint.set_impl(key, value)?;
        }

        checkpoint.flush()?;

        Ok(checkpoint)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()?;
        self.file.get_ref().sync_all()
    }

    fn remove_impl(&mut self, key: Key) -> std::io::Result<()> {
        self.set_impl(key, None) // empty value
    }

    pub fn remove(&mut self, key: Key) -> std::io::Result<()> {
        self.remove_impl(key)?;
        self.flush()
    }

    pub fn to_alist(&mut self) -> std::io::Result<Vec<(Key, Value)>> {
        let keys: Vec<Key> = self.index.keys().cloned().collect();

        keys.into_iter()
            .map(|key| Ok((key.clone(), self.get(&key)?.unwrap())))
            .collect()
    }

    pub fn run_batch(&mut self, batch: &mut Batch) -> std::io::Result<()> {
        use super::batch::Action::{Remove, Set};

        for action in batch.take() {
            match action {
                Set(key, value) => self.set_impl(key, Some(value))?,
                Remove(key) => self.remove_impl(key)?,
            }
        }

        self.flush()
    }

    pub fn gc(&mut self) -> std::io::Result<()> {
        let directory = self.filename.parent().unwrap();
        let mut new_db = Self::create_impl(directory, CreateMode::Temporary)?;

        let keys: Vec<Key> = self.index.keys().cloned().collect();

        for key in keys {
            let value = self.get(&key)?;
            new_db.set_impl(key, value)?;
        }

        new_db.flush()?;

        exchange_file_atomically(&self.filename, &new_db.filename)?;

        new_db.filename = self.filename.clone();
        new_db.uuid = self.uuid.clone();

        *self = new_db;

        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
fn exchange_file_atomically(db_path: &Path, tmp_path: &Path) -> std::io::Result<()> {
    std::fs::rename(tmp_path, db_path).unwrap();

    Ok(())
}

// `renameat2` is a Linux syscall
#[cfg(target_os = "linux")]
fn exchange_file_atomically(db_path: &Path, tmp_path: &Path) -> std::io::Result<()> {
    use std::os::unix::prelude::OsStrExt;

    let cstr_db_path = std::ffi::CString::new(db_path.as_os_str().as_bytes())?;
    let cstr_db_path = cstr_db_path.as_ptr();

    let cstr_tmp_path = std::ffi::CString::new(tmp_path.as_os_str().as_bytes())?;
    let cstr_tmp_path = cstr_tmp_path.as_ptr();

    // Exchange `db_path` with `tmp_path` atomically
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            cstr_tmp_path,
            libc::AT_FDCWD,
            cstr_db_path,
            libc::RENAME_EXCHANGE,
        )
    };

    if result != 0 {
        let error = std::io::Error::last_os_error();
        return Err(error);
    }

    // Remove previous file
    std::fs::remove_file(tmp_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rand::{Fill, Rng};
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering::SeqCst},
    };

    use super::*;

    struct TempDir {
        path: PathBuf,
    }

    static DIRECTORY_NUMBER: AtomicUsize = AtomicUsize::new(0);

    impl TempDir {
        fn new() -> Self {
            let number = DIRECTORY_NUMBER.fetch_add(1, SeqCst);

            let path = loop {
                let directory = format!("/tmp/mina-rocksdb-test-{}", number);
                let path = PathBuf::from(directory);

                if !path.exists() {
                    break path;
                }
            };

            std::fs::create_dir_all(&path).unwrap();

            Self { path }
        }

        fn as_path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            if let Err(e) = std::fs::remove_dir_all(&self.path) {
                eprintln!(
                    "[test] Failed to remove temporary directory {:?}: {:?}",
                    self.path, e
                );
            }
        }
    }

    fn key(s: &str) -> Key {
        Box::<[u8]>::from(s.as_bytes())
    }

    fn value(s: &str) -> Value {
        Box::<[u8]>::from(s.as_bytes())
        // s.as_bytes().to_vec()
    }

    fn sorted_vec(mut vec: Vec<(Key, Value)>) -> Vec<(Key, Value)> {
        vec.sort_by_cached_key(|(k, _)| k.clone());
        vec
    }

    #[test]
    fn test_empty_value() {
        let db_dir = TempDir::new();

        let mut db = Database::create(db_dir.as_path()).unwrap();

        db.set(key("a"), value("abc")).unwrap();
        let v = db.get(&key("a")).unwrap().unwrap();
        assert_eq!(v, value("abc"));

        db.set(key("a"), value("")).unwrap();
        let v = db.get(&key("a")).unwrap().unwrap();
        assert_eq!(v, value(""));
    }

    #[test]
    fn test_persistent_removed_value() {
        let db_dir = TempDir::new();

        let first = {
            let mut db = Database::create(db_dir.as_path()).unwrap();

            db.set(key("abcd"), value("abcd")).unwrap();

            db.set(key("a"), value("abc")).unwrap();
            let v = db.get(&key("a")).unwrap().unwrap();
            assert_eq!(v, value("abc"));

            db.set(key("a"), value("")).unwrap();
            let v = db.get(&key("a")).unwrap().unwrap();
            assert_eq!(v, value(""));

            db.remove(key("a")).unwrap();
            let v = db.get(&key("a")).unwrap();
            assert!(v.is_none());

            sorted_vec(db.to_alist().unwrap())
        };

        assert_eq!(first.len(), 1);

        let second = {
            let mut db = Database::create(db_dir.as_path()).unwrap();
            sorted_vec(db.to_alist().unwrap())
        };

        assert_eq!(first, second);
    }

    #[test]
    fn test_get_batch() {
        let db_dir = TempDir::new();

        let mut db = Database::create(db_dir.as_path()).unwrap();

        let (key1, key2, key3): (Key, Key, Key) = (
            "a".as_bytes().into(),
            "b".as_bytes().into(),
            "c".as_bytes().into(),
        );
        let data: Value = value("test");

        db.set(key1.clone(), data.clone()).unwrap();
        db.set(key3.clone(), data.clone()).unwrap();

        let res = db.get_batch([key1, key2, key3]).unwrap();

        assert_eq!(res[0].as_ref().unwrap(), &data);
        assert!(res[1].is_none());
        assert_eq!(res[2].as_ref().unwrap(), &data);
    }

    fn make_random_key_values(nkeys: usize) -> Vec<(Key, Value)> {
        let mut rng = rand::thread_rng();

        let mut key = [0; 32];

        let mut key_values = HashMap::with_capacity(nkeys);

        while key_values.len() < nkeys {
            let key_length: usize = rng.gen_range(2..=32);
            key[..key_length].try_fill(&mut rng).unwrap();

            let i = Box::<[u8]>::from(key_values.len().to_ne_bytes());
            key_values.insert(Box::<[u8]>::from(&key[..key_length]), i);
        }

        let mut key_values: Vec<(Key, Value)> = key_values.into_iter().collect();
        key_values.sort_by_cached_key(|(k, _)| k.clone());
        key_values
    }

    #[test]
    fn test_persistent() {
        let db_dir = TempDir::new();

        let mut rng = rand::thread_rng();
        let nkeys: usize = rng.gen_range(1000..2000);
        let sorted = make_random_key_values(nkeys);

        let first = {
            let mut db = Database::create(db_dir.as_path()).unwrap();
            db.set_batch(sorted.clone(), []).unwrap();
            let mut alist = db.to_alist().unwrap();
            alist.sort_by_cached_key(|(k, _)| k.clone());
            alist
        };

        assert_eq!(sorted, first);

        let second = {
            let mut db = Database::create(db_dir.as_path()).unwrap();
            let mut alist = db.to_alist().unwrap();
            alist.sort_by_cached_key(|(k, _)| k.clone());
            alist
        };

        assert_eq!(first, second);
    }

    #[test]
    fn test_gc() {
        let db_dir = TempDir::new();

        let mut rng = rand::thread_rng();
        let nkeys: usize = rng.gen_range(1000..2000);
        let sorted = make_random_key_values(nkeys);

        let mut db = Database::create(db_dir.as_path()).unwrap();
        db.set_batch(sorted.clone(), []).unwrap();

        (10..50).for_each(|index| {
            db.remove(sorted[index].0.clone()).unwrap();
        });

        let offset = db.current_file_offset;

        let mut alist1 = db.to_alist().unwrap();
        alist1.sort_by_cached_key(|(k, _)| k.clone());

        db.gc().unwrap();
        assert!(offset > db.current_file_offset);

        let mut alist2 = db.to_alist().unwrap();
        alist2.sort_by_cached_key(|(k, _)| k.clone());
        assert_eq!(alist1, alist2);

        db.set(key("a"), value("b")).unwrap();
        assert_eq!(db.get(&key("a")).unwrap().unwrap(), value("b"));
    }

    #[test]
    fn test_to_alist() {
        let db_dir = TempDir::new();

        let mut rng = rand::thread_rng();

        let nkeys: usize = rng.gen_range(1000..2000);

        let sorted = make_random_key_values(nkeys);

        let mut db = Database::create(db_dir.as_path()).unwrap();

        db.set_batch(sorted.clone(), []).unwrap();

        let mut alist = db.to_alist().unwrap();
        alist.sort_by_cached_key(|(k, _)| k.clone());

        assert_eq!(sorted, alist);
    }

    #[test]
    fn test_checkpoint_read() {
        let db_dir = TempDir::new();

        let mut rng = rand::thread_rng();

        let nkeys: usize = rng.gen_range(1000..2000);

        let sorted = make_random_key_values(nkeys);

        let mut db_hashtbl: HashMap<_, _> = sorted.into_iter().collect();
        let mut cp_hashtbl: HashMap<_, _> = db_hashtbl.clone();

        let mut db = Database::create(db_dir.as_path()).unwrap();

        for (key, data) in &db_hashtbl {
            db.set(key.clone(), data.clone()).unwrap();
        }

        let cp_dir = TempDir::new();
        let mut cp = db.create_checkpoint(cp_dir.as_path()).unwrap();

        db_hashtbl.insert(key("db_key"), value("db_data"));
        cp_hashtbl.insert(key("cp_key"), value("cp_data"));

        db.set(key("db_key"), value("db_data")).unwrap();
        cp.set(key("cp_key"), value("cp_data")).unwrap();

        let db_sorted: Vec<_> = sorted_vec(db_hashtbl.into_iter().collect());
        let cp_sorted: Vec<_> = sorted_vec(cp_hashtbl.into_iter().collect());

        let db_alist = sorted_vec(db.to_alist().unwrap());
        let cp_alist = sorted_vec(cp.to_alist().unwrap());

        assert_eq!(db_sorted, db_alist);
        assert_eq!(cp_sorted, cp_alist);
    }

    #[test]
    fn dump_ondisk_database_keys() {
        // let mut db = Database::create("/tmp/mydir").unwrap();

        // for (k, v) in &[
        //     ("a", "ab"),
        //     ("a1", "ab"),
        //     ("a2", "ab"),
        //     ("a3", "ab"),
        // ] {
        //     db.set(key(k), value(v)).unwrap();
        // }

        let Ok(directory) = std::env::var("DUMP_ONDISK_DIR") else {
            return
        };

        let directory = PathBuf::from(directory);
        let filename = directory.join("db");
        assert!(filename.exists());

        Database::reload(filename).unwrap();
    }
}

use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Seek, SeekFrom, Write},
    path::Path,
};

use super::batch::Batch;

pub(super) type Key = Box<[u8]>;
pub(super) type Value = Vec<u8>;
pub(super) type Offset = u64;

pub type Uuid = String;

const KEY_REMOVED: u32 = u32::MAX;

pub struct Database {
    uuid: Uuid,
    index: HashMap<Key, Offset>,

    /// Points to end of file
    current_file_offset: Offset,
    file: BufWriter<std::fs::File>,

    buffer: RefCell<Option<Vec<u8>>>,
}

struct Header {
    key_length: u32,
    value_length: u32,
}

impl Header {
    pub const NBYTES: usize = 8;

    fn key_length(&self) -> u64 {
        self.key_length as u64
    }

    fn value_length(&self) -> u64 {
        self.value_length as u64
    }

    fn value_offset(&self, header_offset: Offset) -> Offset {
        header_offset
            .checked_add(self.key_length())
            .unwrap()
            .checked_add(Header::NBYTES as u64)
            .unwrap()
    }
}

pub fn next_uuid() -> Uuid {
    uuid::Uuid::new_v4().to_string()
}

fn read_u32(slice: &[u8]) -> std::io::Result<u32> {
    slice
        .get(..4)
        .and_then(|slice: &[u8]| slice.try_into().ok())
        .map(u32::from_le_bytes)
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

impl Database {
    pub fn create(directory: impl AsRef<Path>) -> std::io::Result<Self> {
        let directory = directory.as_ref();

        let filename = directory.join("db");

        if filename.try_exists()? {
            return Self::reload(&filename);
        }

        if !directory.try_exists()? {
            std::fs::create_dir_all(directory)?;
        }

        let file = std::fs::File::options()
            .read(true)
            .write(true)
            .append(true)
            .create_new(true)
            .open(filename)?;

        Ok(Self {
            uuid: next_uuid(),
            index: HashMap::with_capacity(128),
            current_file_offset: 0,
            file: BufWriter::with_capacity(4 * 1024 * 1024, file), // 4 MB
            buffer: RefCell::new(Some(Vec::with_capacity(4096))),
        })
    }

    fn reload(filename: &Path) -> std::io::Result<Self> {
        use std::io::Read;

        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .append(true)
            .create_new(false)
            .open(filename)?;

        let mut offset = 0;
        let end = file.seek(SeekFrom::End(0))?;

        file.seek(SeekFrom::Start(0))?;

        let mut reader = BufReader::with_capacity(4096, file);
        let mut bytes = vec![0; 4096];

        let mut index = HashMap::with_capacity(256);

        while offset < end {
            let header_offset = offset;

            reader.read_exact(&mut bytes[..Header::NBYTES])?;

            let key_length = read_u32(&bytes[..4])? as usize;
            let value_length = read_u32(&bytes[4..])? as usize;

            ensure_buffer_length(&mut bytes, key_length + value_length);

            reader.read_exact(&mut bytes[..key_length + value_length])?;

            let key = Box::<[u8]>::from(&bytes[..key_length]);

            index.insert(key, header_offset);

            offset += (Header::NBYTES + key_length + value_length) as u64;
        }

        if end != offset {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }

        Ok(Self {
            uuid: next_uuid(),
            index,
            current_file_offset: end,
            file: BufWriter::with_capacity(4 * 1024 * 1024, reader.into_inner()), // 4 MB
            buffer: RefCell::new(Some(Vec::with_capacity(4096))),
        })
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn close(&self) {
        todo!()
    }

    fn with_buffer<F, R>(&self, fun: F) -> std::io::Result<R>
    where
        F: FnOnce(&Self, &mut Vec<u8>) -> std::io::Result<R>,
    {
        let mut buffer = self
            .buffer
            .borrow_mut()
            .take()
            .unwrap_or_else(|| Vec::with_capacity(4096));
        buffer.clear();

        let result = fun(self, &mut buffer)?;

        *self.buffer.borrow_mut() = Some(buffer);
        Ok(result)
    }

    fn with_buffer_mut<F, R>(&mut self, fun: F) -> std::io::Result<R>
    where
        F: FnOnce(&mut Self, &mut Vec<u8>) -> std::io::Result<R>,
    {
        let mut buffer = self
            .buffer
            .borrow_mut()
            .take()
            .unwrap_or_else(|| vec![0; 4096]);

        let result = fun(self, &mut buffer);

        *self.buffer.borrow_mut() = Some(buffer);

        result
    }

    fn read_header(&mut self, header_offset: Offset) -> std::io::Result<Header> {
        self.with_buffer_mut(|this, buffer| {
            ensure_buffer_length(buffer, Header::NBYTES);

            read_exact_at(
                this.file.get_mut(),
                &mut buffer[..Header::NBYTES],
                header_offset,
            )?;

            let key_length = read_u32(&buffer[0..])?;
            let value_length = read_u32(&buffer[4..])?;

            Ok(Header {
                key_length,
                value_length,
            })
        })
    }

    fn read_value(&mut self, offset: Offset, length: usize) -> std::io::Result<Value> {
        self.with_buffer_mut(|this, buffer| {
            ensure_buffer_length(buffer, length);

            read_exact_at(this.file.get_mut(), &mut buffer[..length], offset)?;

            Ok(Vec::from(&buffer[..length]))
        })
    }

    /// `&mut self` is required for `File::seek`
    pub fn get(&mut self, key: &[u8]) -> std::io::Result<Option<Value>> {
        let header_offset = match self.index.get(key.as_ref()).copied() {
            Some(header_offset) => header_offset,
            None => return Ok(None),
        };

        let header = self.read_header(header_offset)?;

        let value_offset = header.value_offset(header_offset);
        let value_length = header.value_length() as usize;

        self.read_value(value_offset, value_length).map(Some)
    }

    fn set_impl(&mut self, key: Key, value: Option<Value>) -> std::io::Result<()> {
        let is_removed = value.is_none();

        let value = match value.as_ref() {
            Some(value) => value.as_slice(),
            None => &[],
        };

        let header = Header {
            key_length: key.len().try_into().unwrap(),
            value_length: if is_removed {
                KEY_REMOVED
            } else {
                let length = value.len().try_into().unwrap();
                if length == KEY_REMOVED {
                    return Err(std::io::ErrorKind::InvalidData.into());
                }
                length
            },
        };

        let header_offset = self.current_file_offset;

        self.file.write_all(&header.key_length.to_le_bytes())?;
        self.file.write_all(&header.value_length.to_le_bytes())?;
        self.file.write_all(&key)?;
        self.file.write_all(value)?;

        let buffer_len = Header::NBYTES + key.len() + value.len();
        self.current_file_offset += buffer_len as u64;

        // Update index
        if is_removed {
            // Value is empty, we remove the key from our index
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

        let keys: Vec<_> = self.index.keys().cloned().collect();

        for key in keys {
            let value = self.get(&key)?.unwrap();
            checkpoint.set_impl(key, Some(value))?;
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
        let keys: Vec<_> = self.index.keys().cloned().collect();

        keys.into_iter()
            .map(|key| Ok((key.clone(), self.get(&key)?.unwrap())))
            .collect()
    }

    pub fn run_batch(&mut self, batch: &mut Batch) {
        use super::batch::Action::{Remove, Set};

        for action in batch.take() {
            match action {
                Set(key, value) => self.set_impl(key, Some(value)).unwrap(),
                Remove(key) => self.remove_impl(key).unwrap(),
            }
        }

        self.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use rand::{Fill, Rng};
    use std::path::PathBuf;

    use super::*;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(directory: impl AsRef<Path>) -> std::io::Result<Self> {
            let path = PathBuf::from(directory.as_ref());

            std::fs::create_dir_all(&path)?;

            Ok(Self { path })
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
        s.as_bytes().to_vec()
    }

    #[test]
    fn test_empty_value() {
        let db_dir = TempDir::new("/tmp/mina-rocksdb-test").unwrap();

        let mut db = Database::create(db_dir.as_path()).unwrap();

        db.set(key("a"), value("abc")).unwrap();
        let v = db.get(&key("a")).unwrap().unwrap();
        assert_eq!(v, value("abc"));

        db.set(key("a"), value("")).unwrap();
        let v = db.get(&key("a")).unwrap().unwrap();
        assert_eq!(v, value(""));
    }

    #[test]
    fn test_get_batch() {
        let db_dir = TempDir::new("/tmp/mina-rocksdb-test").unwrap();

        let mut db = Database::create(db_dir.as_path()).unwrap();

        let (key1, key2, key3): (Key, Key, Key) = (
            "a".as_bytes().into(),
            "b".as_bytes().into(),
            "c".as_bytes().into(),
        );
        let data: Value = "test".as_bytes().to_vec();

        db.set(key1.clone(), data.clone()).unwrap();
        db.set(key3.clone(), data.clone()).unwrap();

        let res = db.get_batch([key1, key2, key3]).unwrap();

        assert_eq!(res[0].as_ref().unwrap(), data.as_slice());
        assert!(res[1].is_none());
        assert_eq!(res[2].as_ref().unwrap(), data.as_slice());
    }

    fn make_random_key_values(nkeys: usize) -> Vec<(Key, Value)> {
        let mut rng = rand::thread_rng();

        let mut key = [0; 32];

        let mut key_values = HashMap::with_capacity(nkeys);

        while key_values.len() < nkeys {
            let key_length: usize = rng.gen_range(2..=32);
            key[..key_length].try_fill(&mut rng).unwrap();

            let i = key_values.len().to_ne_bytes().to_vec();
            key_values.insert(Box::<[u8]>::from(&key[..key_length]), i);
        }

        let mut key_values: Vec<(Key, Value)> = key_values.into_iter().collect();
        key_values.sort_by_cached_key(|(k, _)| k.clone());
        key_values
    }

    #[test]
    fn test_persistent() {
        let db_dir = TempDir::new("/tmp/mina-rocksdb-test").unwrap();

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
    fn test_to_alist() {
        let db_dir = TempDir::new("/tmp/mina-rocksdb-test").unwrap();

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
        let sorted_vec = |mut vec: Vec<(Key, Value)>| {
            vec.sort_by_cached_key(|(k, _)| k.clone());
            vec
        };

        let db_dir = TempDir::new("/tmp/test-cp").unwrap();

        let mut rng = rand::thread_rng();

        let nkeys: usize = rng.gen_range(1000..2000);

        let sorted = make_random_key_values(nkeys);

        let mut db_hashtbl: HashMap<_, _> = sorted.into_iter().collect();
        let mut cp_hashtbl: HashMap<_, _> = db_hashtbl.clone();

        let mut db = Database::create(db_dir.as_path()).unwrap();

        for (key, data) in &db_hashtbl {
            db.set(key.clone(), data.clone()).unwrap();
        }

        let cp_dir = TempDir::new("/tmp/test-cp2").unwrap();
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
}

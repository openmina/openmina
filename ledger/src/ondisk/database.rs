use std::{cell::RefCell, collections::HashMap, fs::File, os::unix::prelude::FileExt, path::Path};

use uuid::Uuid;

type Key = Box<[u8]>;
type Value = Vec<u8>;
type Offset = u64;

struct Database {
    uuid: Uuid,
    index: HashMap<Key, Offset>,

    file_offset: Offset,
    file: std::fs::File,

    buffer: RefCell<Option<Vec<u8>>>,
}

// key size (u32)
// value size (u32)
// key
// value

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

fn read_u32(slice: &[u8]) -> std::io::Result<u32> {
    slice
        .get(0..4)
        .and_then(|slice: &[u8]| slice.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or(std::io::ErrorKind::UnexpectedEof.into())
}

fn ensure_buffer_length(buffer: &mut Vec<u8>, length: usize) {
    let capacity = buffer.capacity();

    if capacity < length {
        buffer.reserve(length - capacity);
    }
}

impl Database {
    pub fn create(directory: &Path) -> std::io::Result<Self> {
        let file = std::fs::File::options()
            .write(true)
            .append(true)
            .create_new(true)
            .open(directory)?;

        Ok(Self {
            uuid: Uuid::new_v4(),
            index: HashMap::with_capacity(128),
            file_offset: 0,
            file,
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
            .unwrap_or_else(|| Vec::with_capacity(4096));
        buffer.clear();

        let result = fun(self, &mut buffer)?;

        *self.buffer.borrow_mut() = Some(buffer);
        Ok(result)
    }

    fn read_header(&self, header_offset: Offset) -> std::io::Result<Header> {
        self.with_buffer(|this, buffer| {
            this.file
                .read_exact_at(&mut buffer[..Header::NBYTES], header_offset)?;

            let key_length = read_u32(&buffer[0..])?;
            let value_length = read_u32(&buffer[4..])?;

            Ok(Header {
                key_length,
                value_length,
            })
        })
    }

    fn read_value(&self, offset: Offset, length: usize) -> std::io::Result<Value> {
        self.with_buffer(|this, buffer| {
            ensure_buffer_length(buffer, length);

            this.file.read_exact_at(&mut buffer[..length], offset)?;

            Ok(Vec::from(&buffer[..length]))
        })
    }

    pub fn get(&self, key: Key) -> std::io::Result<Option<Value>> {
        let header_offset = match self.index.get(&key).copied() {
            Some(header_offset) => header_offset,
            None => return Ok(None),
        };

        let header = self.read_header(header_offset)?;

        let value_offset = header.value_offset(header_offset);
        let value_length = header.value_length() as usize;

        self.read_value(value_offset, value_length).map(Some)
    }

    fn set_impl(&mut self, key: Key, value: Value) -> std::io::Result<()> {
        use std::io::Write;

        let header = Header {
            key_length: key.len().try_into().unwrap(),
            value_length: value.len().try_into().unwrap(),
        };

        let value_is_empty = value.is_empty();

        let header_offset = self.with_buffer_mut(|this, buffer| {
            buffer.write_all(&header.key_length.to_le_bytes())?;
            buffer.write_all(&header.value_length.to_le_bytes())?;
            buffer.write_all(&key)?;
            buffer.write_all(&value)?;

            let offset = this.file_offset;
            let buffer_len = buffer.len();

            this.file.write_all(buffer)?;

            this.file_offset += buffer_len as u64;

            Ok(offset)
        })?;

        if value_is_empty {
            self.index.remove(&key);
        } else {
            self.index.insert(key, header_offset);
        }

        Ok(())
    }

    pub fn set(&mut self, key: Key, value: Value) -> std::io::Result<()> {
        self.set_impl(key, value)?;
        self.file.sync_all()?;
        Ok(())
    }

    pub fn set_batch(
        &mut self,
        key_data_pairs: impl Iterator<Item = (Key, Value)>,
        remove_keys: impl Iterator<Item = Key>,
    ) -> std::io::Result<()> {
        for (key, value) in key_data_pairs {
            self.set_impl(key, value)?;
        }

        for key in remove_keys {
            self.set_impl(key, Vec::new())? // empty value
        }

        self.file.sync_all()?;

        Ok(())
    }
}

// let create_checkpoint t dir =
//   Rocks.checkpoint_create t.db ~dir ?log_size_for_flush:None () ;
//   create dir

// let make_checkpoint t dir =
//   Rocks.checkpoint_create t.db ~dir ?log_size_for_flush:None ()

// let get_uuid t = t.uuid

// let close t = Rocks.close t.db

// let get t ~(key : Bigstring.t) : Bigstring.t option =
//   Rocks.get ?pos:None ?len:None ?opts:None t.db key

// let get_batch t ~(keys : Bigstring.t list) : Bigstring.t option list =
//   Rocks.multi_get t.db keys

// let set t ~(key : Bigstring.t) ~(data : Bigstring.t) : unit =
//   Rocks.put ?key_pos:None ?key_len:None ?value_pos:None ?value_len:None
//     ?opts:None t.db key data

// let set_batch t ?(remove_keys = [])
//     ~(key_data_pairs : (Bigstring.t * Bigstring.t) list) : unit =
//   let batch = Rocks.WriteBatch.create () in
//   (* write to batch *)
//   List.iter key_data_pairs ~f:(fun (key, data) ->
//       Rocks.WriteBatch.put batch key data ) ;
//   (* Delete any key pairs *)
//   List.iter remove_keys ~f:(fun key -> Rocks.WriteBatch.delete batch key) ;
//   (* commit batch *)
//   Rocks.write t.db batch

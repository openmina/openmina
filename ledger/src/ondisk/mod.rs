//! # Database
//!
//! Database is a lightweight, single append-only file, key-value store designed as an
//! alternative to RocksDB.
//!
//! ## Storage Format
//!
//! Each entry in the Database has the following structure:
//!
//! ```ignored
//! +------------+-----------+-----------+
//! |    HDR     |    KEY    |   VALUE   |
//! | (23 bytes) | (X bytes) | (X bytes) |
//! +------------+-----------+-----------+
//!       ^
//!       |
//!       |
//! +------------+----------+--------------+----------+-----------+------------+
//! | KEY_LENGTH |   KIC    | VALUE_LENGTH |   VIC    | RM_FLAG   |   CRC32    |
//! | (8 bytes)  | (1 byte) | (8 bytes)    | (1 byte) | (1 byte)  | (4 bytes)  |
//! +------------+----------+--------------+----------+-----------+------------+
//! ```
//!
//! Where:
//! - `HDR`: A 23 bytes header
//!   - `key length`: Length of the key, stored in 8 bytes.
//!   - `key_is_compressed (KIC)`: A flag indicating if the key is compressed, stored in 1 byte.
//!   - `value length`: Length of the value, stored in 8 bytes.
//!   - `value_is_compressed (VIC)`: A flag indicating if the value is compressed, stored in 1 byte.
//!   - `is_removed (RM_FLAG)`: A flag indicating if the entry has been removed, stored in 1 byte.
//!   - `crc32`: The CRC32 checksum of the entry (including its header), stored in 4 bytes.
//! - `KEY`: The key data
//! - `VALUE`: The value data
//!
//! ## Example Usage
//!
//! Create an instance of MyDatabase:
//!
//! ```rust
//! # use mina_tree::ondisk::Database;
//! # fn usage() -> std::io::Result<()> {
//! # let k = |s: &str| Box::<[u8]>::from(s.as_bytes());
//! # let v = k;
//! let mut db = Database::create("/tmp/my_database")?;
//!
//! // Insert a key-value pair:
//! db.set(k("key1"), v("value1"))?;
//!
//! // Retrieve a value by key:
//! let result = db.get(&k("key1"))?;
//! assert_eq!(result, Some(v("value1")));
//! # Ok(())
//! # }
//! ```

pub mod batch;
pub(self) mod compression;
mod database;
pub(self) mod lock;

pub use batch::*;
pub use database::*;

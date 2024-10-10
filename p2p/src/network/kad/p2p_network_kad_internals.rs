use std::{
    fmt::Debug,
    ops::{Add, Shr, Sub},
};

use crypto_bigint::{ArrayEncoding, Encoding, U256};
use derive_more::From;
use libp2p_identity::DecodingError;
use multiaddr::Multiaddr;
use openmina_core::bug_condition;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    ConnectionType, P2pNetworkKademliaMultiaddrError, P2pNetworkKademliaPeerIdError, PeerId,
};

use super::CID;

mod u256_serde {
    use std::array::TryFromSliceError;

    use crypto_bigint::{Encoding, U256};
    use serde::{Deserialize, Serialize};

    pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let hex = hex::encode(value.to_be_bytes());
            serializer.serialize_str(&hex)
        } else {
            value.serialize(serializer)
        }
    }

    fn decode_error<E: serde::de::Error>(e: hex::FromHexError) -> E {
        E::custom(format!("error converting from hex string: {e}"))
    }

    fn bytes_error<E: serde::de::Error>(e: TryFromSliceError) -> E {
        E::custom(format!("error converting from slice to array: {e}"))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = hex::decode(s)
                .map_err(decode_error)?
                .as_slice()
                .try_into()
                .map_err(bytes_error)?;
            let u256 = U256::from_be_bytes(bytes);
            Ok(u256)
        } else {
            U256::deserialize(deserializer)
        }
    }
}

/// Kademlia key, sha256 of the node's peer id.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct P2pNetworkKadKey(#[serde(with = "u256_serde")] U256);

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize, thiserror::Error)]
pub enum P2pNetworkKadKeyError {
    #[error("decoding error")]
    DecodingError,
}

impl TryFrom<&PeerId> for P2pNetworkKadKey {
    type Error = P2pNetworkKadKeyError;

    fn try_from(value: &PeerId) -> Result<Self, Self::Error> {
        P2pNetworkKadKey::try_from(*value)
    }
}

impl TryFrom<PeerId> for P2pNetworkKadKey {
    type Error = P2pNetworkKadKeyError;

    fn try_from(value: PeerId) -> Result<Self, Self::Error> {
        Ok(P2pNetworkKadKey::from(
            CID::try_from(value).map_err(|_| P2pNetworkKadKeyError::DecodingError)?,
        ))
    }
}

impl From<CID> for P2pNetworkKadKey {
    fn from(value: CID) -> Self {
        P2pNetworkKadKey(U256::from_be_byte_array(Sha256::digest(value.0)))
    }
}

impl Debug for P2pNetworkKadKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            let bytes = self.0.to_be_bytes();
            f.write_str(&hex::encode(bytes))
        } else {
            f.debug_tuple("P2pNetworkKadKey").field(&self.0).finish()
        }
    }
}

impl Add<&P2pNetworkKadDist> for &P2pNetworkKadKey {
    type Output = P2pNetworkKadKey;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: &P2pNetworkKadDist) -> Self::Output {
        P2pNetworkKadKey(self.0 ^ rhs.0)
    }
}

impl Sub for P2pNetworkKadKey {
    type Output = P2pNetworkKadDist;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self::Output {
        P2pNetworkKadDist(self.0 ^ rhs.0)
    }
}

impl Sub<&P2pNetworkKadKey> for &P2pNetworkKadKey {
    type Output = P2pNetworkKadDist;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: &P2pNetworkKadKey) -> Self::Output {
        P2pNetworkKadDist(self.0 ^ rhs.0)
    }
}

impl Sub<P2pNetworkKadKey> for &P2pNetworkKadKey {
    type Output = P2pNetworkKadDist;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: P2pNetworkKadKey) -> Self::Output {
        P2pNetworkKadDist(self.0 ^ rhs.0)
    }
}

impl Sub<&P2pNetworkKadKey> for P2pNetworkKadKey {
    type Output = P2pNetworkKadDist;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: &P2pNetworkKadKey) -> Self::Output {
        P2pNetworkKadDist(self.0 ^ rhs.0)
    }
}

impl Shr<usize> for &P2pNetworkKadKey {
    type Output = P2pNetworkKadKey;

    fn shr(self, rhs: usize) -> Self::Output {
        P2pNetworkKadKey(self.0 >> rhs)
    }
}

/// Kademlia distance between two nodes, calculated as `XOR` of their keys.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct P2pNetworkKadDist(#[serde(with = "u256_serde")] U256);

impl Debug for P2pNetworkKadDist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            let bytes = self.0.to_be_bytes();
            f.write_str(&hex::encode(bytes))
        } else {
            f.debug_tuple("P2pNetworkKadDist").field(&self.0).finish()
        }
    }
}

impl P2pNetworkKadDist {
    /// Returns (maximal possible) index of the bucket for this distance. In
    /// other words, this is the length of the common prefix of two nodes
    /// withing this distance from each other.
    fn to_index(&self) -> usize {
        256 - self.0.bits_vartime()
    }
}

/// Converts a K-bucket index to distance as `2^n - 1`. For `i`, any node with its keys
/// having at least `i` highest bits in common are within this distance.
impl From<usize> for P2pNetworkKadDist {
    fn from(value: usize) -> Self {
        P2pNetworkKadDist(U256::MAX >> value)
    }
}

/// Kademlia routing table, with `K` parameter, the maximum number of records
/// for each bucket. Usually it is set to `20`.
#[derive(Clone, Serialize, Deserialize)]
pub struct P2pNetworkKadRoutingTable<const K: usize = 20> {
    /// SHA256 of the current node's id.
    pub this_key: P2pNetworkKadKey,
    /// Kademlia K-buckets. Under index `i` located elements within distance
    /// `2^(256-i)` from the current node at most. If there is also `i+1` (i.e.
    /// `i` is not the last index), then distance from the current node to the
    /// elements under `i` are greater than `2^(256-i-1)`.
    pub buckets: Vec<P2pNetworkKadBucket<K>>,
}

impl<const K: usize> Default for P2pNetworkKadRoutingTable<K> {
    fn default() -> Self {
        Self {
            this_key: P2pNetworkKadKey(U256::ZERO),
            buckets: Default::default(),
        }
    }
}

impl<const K: usize> Debug for P2pNetworkKadRoutingTable<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            if f.sign_plus() {
                writeln!(f, "this key: {:#?}", self.this_key)?;
            }
            for (i, bucket) in self.buckets.iter().enumerate() {
                writeln!(f, "{i}: {}", bucket.0.len())?;
                if f.sign_plus() {
                    for entry in &bucket.0 {
                        writeln!(f, "    {:#.*?}", i, &entry.key)?;
                    }
                }
            }
            Ok(())
        } else {
            f.debug_struct("P2pNetworkKadRoutingTable")
                .field("this_key", &self.this_key)
                .field("buckets", &self.buckets)
                .finish()
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("bucket capacity exceeded")]
pub struct P2pNetworkKadRoutingTableInsertError;

impl<const K: usize> P2pNetworkKadRoutingTable<K> {
    pub fn new(this_entry: P2pNetworkKadEntry) -> Self {
        let this_key = this_entry.key.clone();
        let global_bucket = P2pNetworkKadBucket(vec![this_entry]);
        let buckets = vec![global_bucket];
        P2pNetworkKadRoutingTable { this_key, buckets }
    }

    /// Inserts the `entry` into the routing table.
    ///
    /// Returns `Ok(true)` it is added as a new entry, and `Ok(false)` if an
    /// existing one has been updated.
    ///
    /// If there is no space for adding the new entry (i.e. corresponding
    /// K-bucket is full and cannot be split), then `Err(_)` value is returned.
    ///
    /// TODO: should it be just another variant in `Ok(_)`?
    ///
    /// Filters `entry.addrs` for supported addresses if non of addresses are supported returns `Err(_)`
    pub fn insert(
        &mut self,
        entry: P2pNetworkKadEntry,
    ) -> Result<bool, P2pNetworkKadRoutingTableInsertError> {
        if entry.addrs.is_empty() {
            return Err(P2pNetworkKadRoutingTableInsertError);
        }
        // distance to this node
        let dist = &self.this_key - &entry.key;

        // index of the closest k-bucket that can contain this node.
        let index = dist.to_index();

        let mut max_index = self.buckets.len() - 1;
        loop {
            if index < max_index {
                // bucket cannot be split
                if self.buckets[index].can_insert(&entry) {
                    break Ok(self.buckets[index].insert(entry));
                } else {
                    break Err(P2pNetworkKadRoutingTableInsertError);
                }
            } else if self.buckets[max_index].can_insert(&entry) {
                break Ok(self.buckets[max_index].insert(entry));
            } else {
                max_index += 1;
                let split_dist = max_index.into();
                let Some((bucket1, bucket2)) = self
                    .buckets
                    .pop()
                    .map(|b| b.split(|e| (&self.this_key - &e.key) >= split_dist))
                else {
                    bug_condition!("should be unreachable");
                    return Err(P2pNetworkKadRoutingTableInsertError);
                };
                self.buckets.extend([bucket1, bucket2]);
            }
        }
    }

    /// Looks up a Kademlia entry with the specified `key`.
    pub fn look_up(&self, key: &P2pNetworkKadKey) -> Option<&P2pNetworkKadEntry> {
        // distance to this node
        let dist = &self.this_key - key;

        // index of the closest k-bucket that can contain this node.
        let index = dist.to_index().min(self.buckets.len() - 1);

        self.buckets[index].iter().find(|e| &e.key == key)
    }

    /// FIND_NODE backend. Returns iterator of nodes closest to the specified
    /// `key`, excluding nodes that correspond to the `key` itself and
    /// `self.this_key`.
    ///
    /// The number of entries is limited to 20.
    pub fn find_node<'a>(
        &'a self,
        key: &'a P2pNetworkKadKey,
    ) -> impl Iterator<Item = &'a P2pNetworkKadEntry> {
        self.closest_peers(key).take(20)
    }

    /// Returns iterator of nodes closest to the current one.
    /// TODO: use reverse order over bucket, from the deepest bucket.
    pub fn closest_peers<'a>(&'a self, key: &'a P2pNetworkKadKey) -> ClosestPeers<'a, K> {
        ClosestPeers::new(self, key)
    }

    #[cfg(test)]
    fn assert_k_buckets(&self) {
        let mut prev_dist = None;
        for (i, bucket) in self.buckets.iter().enumerate().rev() {
            assert!(bucket.0.len() <= K, "{self:+#?}");
            let dist = P2pNetworkKadDist::from(i);
            for entry in &bucket.0 {
                assert!(
                    &self.this_key - &entry.key <= dist,
                    "for {:#?} at {i} distance {:#?} is too big, expecting at most {dist:#?}\nrouting table:\n{self:+#?}",
                    entry.key,
                    &self.this_key - &entry.key,
                );
                if let Some(prev_dist) = &prev_dist {
                    assert!(
                        &(&self.this_key - &entry.key) > prev_dist,
                        "distance too small: {:#?}\nrouting table:\n{:+#?}\ndist: {:#?}\nprev_dist: {:#?}",
                        entry.key,
                        self,
                        &self.this_key - &entry.key,
                        prev_dist,
                    );
                }
            }
            prev_dist = Some(dist);
        }
    }
}

impl Extend<P2pNetworkKadEntry> for P2pNetworkKadRoutingTable {
    fn extend<T: IntoIterator<Item = P2pNetworkKadEntry>>(&mut self, iter: T) {
        for entry in iter {
            // TODO(akoptelov): log addition status?
            let _ = self.insert(entry);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct P2pNetworkKadEntry {
    pub key: P2pNetworkKadKey,
    pub peer_id: PeerId,
    addrs: Vec<Multiaddr>,
    pub connection: ConnectionType,
}

impl P2pNetworkKadEntry {
    pub const MAX_ADDRS: usize = 16;

    pub fn new(peer_id: PeerId, addrs: Vec<Multiaddr>) -> Result<Self, P2pNetworkKadKeyError> {
        if addrs.len() > Self::MAX_ADDRS {
            openmina_core::log::info!(
                openmina_core::log::system_time();
                kind = "P2pNetworkKadEntry new",
                summary = format!("truncating {addrs:?} to {} elements", Self::MAX_ADDRS),
            );
        }

        Ok(P2pNetworkKadEntry {
            key: peer_id.try_into()?,
            peer_id,
            addrs: addrs.into_iter().take(Self::MAX_ADDRS).collect(),
            connection: ConnectionType::NotConnected,
        })
    }

    pub fn dist(&self, other: &P2pNetworkKadEntry) -> P2pNetworkKadDist {
        &self.key - &other.key
    }

    pub fn addresses(&self) -> &Vec<Multiaddr> {
        &self.addrs
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize, thiserror::Error)]
pub enum P2pNetworkKadEntryTryFromError {
    #[error(transparent)]
    PeerId(#[from] P2pNetworkKademliaPeerIdError),
    #[error(transparent)]
    Key(#[from] P2pNetworkKadKeyError),
    #[error(transparent)]
    Multiaddr(#[from] P2pNetworkKademliaMultiaddrError),
}

impl TryFrom<super::mod_Message::Peer<'_>> for P2pNetworkKadEntry {
    type Error = P2pNetworkKadEntryTryFromError;

    fn try_from(value: super::mod_Message::Peer) -> Result<Self, Self::Error> {
        let peer_id = super::peer_id_try_from_bytes(value.id)?;
        let key = peer_id.try_into()?;
        let addrs = value
            .addrs
            .into_iter()
            .map(super::multiaddr_try_from_bytes)
            .collect::<Result<_, _>>()?;
        let connection = value.connection.into();
        Ok(P2pNetworkKadEntry {
            peer_id,
            key,
            addrs,
            connection,
        })
    }
}

impl<'a> TryFrom<&'a P2pNetworkKadEntry> for super::mod_Message::Peer<'a> {
    type Error = DecodingError;

    fn try_from(value: &'a P2pNetworkKadEntry) -> Result<Self, Self::Error> {
        Ok(super::mod_Message::Peer {
            id: (&value.peer_id).try_into()?,
            addrs: value
                .addrs
                .iter()
                .map(|addr| addr.as_ref().into())
                .collect(),
            connection: value.connection.into(),
        })
    }
}

pub struct ClosestPeers<'a, const K: usize> {
    table: &'a P2pNetworkKadRoutingTable<K>,
    index_iter: std::vec::IntoIter<usize>,
    bucket_index: usize,
    bucket_iterator: std::vec::IntoIter<&'a P2pNetworkKadEntry>,
    key: &'a P2pNetworkKadKey,
}

impl<'a, const K: usize> ClosestPeers<'a, K> {
    fn new(table: &'a P2pNetworkKadRoutingTable<K>, key: &'a P2pNetworkKadKey) -> Self {
        let dist = &table.this_key - key;
        let mut index_iter = Self::bucket_index_iterator(dist, table.buckets.len());
        let bucket_index = index_iter
            .next()
            .expect("implementation should ensure there is at least one bucket");
        let bucket_iterator =
            Self::get_bucket_iter(&table.buckets[bucket_index], key, &table.this_key);
        // println!(">>> first bucket {}", bucket_index);
        ClosestPeers {
            table,
            index_iter,
            bucket_index,
            bucket_iterator,
            key,
        }
    }

    fn bucket_index_iterator(
        dist: P2pNetworkKadDist,
        buckets_len: usize,
    ) -> std::vec::IntoIter<usize> {
        let (mut ones, zeroes) =
            (0..buckets_len).partition::<Vec<_>, _>(|index| dist.0.bit_vartime(255 - *index));
        ones.extend(zeroes.into_iter().rev());
        let it: std::vec::IntoIter<usize> = ones.into_iter();
        it
    }

    fn get_bucket_iter(
        bucket: &'a P2pNetworkKadBucket<K>,
        key: &P2pNetworkKadKey,
        this_key: &P2pNetworkKadKey,
    ) -> std::vec::IntoIter<&'a P2pNetworkKadEntry> {
        let mut vec = Vec::from_iter(
            bucket
                .into_iter()
                .filter(|e| &e.key != key && &e.key != this_key),
        );
        vec.sort_by_cached_key(|entry| key - &entry.key);
        vec.into_iter()
    }
}

impl<'a, const K: usize> Iterator for ClosestPeers<'a, K> {
    type Item = &'a P2pNetworkKadEntry;

    fn next(&mut self) -> Option<Self::Item> {
        Some(loop {
            if let Some(item) = self.bucket_iterator.next() {
                break item;
            }
            self.bucket_index = self.index_iter.next()?;
            self.bucket_iterator = Self::get_bucket_iter(
                &self.table.buckets[self.bucket_index],
                self.key,
                &self.table.this_key,
            );
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, From)]
pub struct P2pNetworkKadBucket<const K: usize>(Vec<P2pNetworkKadEntry>);

impl<const K: usize> P2pNetworkKadBucket<K> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, P2pNetworkKadEntry> {
        self.0.iter()
    }

    /// Checks if the `entry` can be inserted into this bucket, that is, either
    /// there is a free slot, or the entry with this peer_id already there and
    /// only needs to be updated.
    fn can_insert(&self, entry: &P2pNetworkKadEntry) -> bool {
        self.len() < K || self.0.iter().any(|e| e.key == entry.key)
    }

    /// Inserts an entry into the bucket. Returns true if a new entry is added,
    /// false if an existing one is updated.
    fn insert(&mut self, entry: P2pNetworkKadEntry) -> bool {
        if let Some(pos) = self.0.iter().position(|e| e.key == entry.key) {
            let e = &mut self.0[pos];

            if e.peer_id != entry.peer_id {
                bug_condition!(
                    "Kad entry peer_id mismatch {:?} != {:?}",
                    e.peer_id,
                    entry.peer_id
                );
            }

            for addr in entry.addrs {
                if e.addrs.len() >= P2pNetworkKadEntry::MAX_ADDRS {
                    openmina_core::warn!(
                        openmina_core::log::system_time();
                        kind = "P2pNetworkKadBucket insert",
                        summary = format!("Skipping updates to Kad entry multiaddress list"),
                    );
                    break;
                }

                if !e.addrs.contains(&addr) {
                    e.addrs.push(addr);
                }
            }
            false
        } else {
            if self.len() >= K {
                bug_condition!("Kad bucket len {:?} >= K ({:?})", self.len(), K);
            }
            self.0.push(entry);
            true
        }
    }

    /// Splits this bucket into two, keeping entries that are not closer to the
    /// current node than the `dist`.
    fn split<F: Fn(&P2pNetworkKadEntry) -> bool>(self, f: F) -> (Self, Self) {
        self.into_iter().partition(f)
    }
}

impl<const K: usize> FromIterator<P2pNetworkKadEntry> for P2pNetworkKadBucket<K> {
    fn from_iter<T: IntoIterator<Item = P2pNetworkKadEntry>>(iter: T) -> Self {
        P2pNetworkKadBucket(Vec::from_iter(iter))
    }
}

impl<const K: usize> IntoIterator for P2pNetworkKadBucket<K> {
    type Item = P2pNetworkKadEntry;

    type IntoIter = <Vec<P2pNetworkKadEntry> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const K: usize> Extend<P2pNetworkKadEntry> for P2pNetworkKadBucket<K> {
    fn extend<T: IntoIterator<Item = P2pNetworkKadEntry>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl<'a, const K: usize> IntoIterator for &'a P2pNetworkKadBucket<K> {
    type Item = &'a P2pNetworkKadEntry;

    type IntoIter = std::slice::Iter<'a, P2pNetworkKadEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.as_slice().iter()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crypto_bigint::{Random, U256};
    use multiaddr::multiaddr;

    use crate::{identity::SecretKey, PeerId, CID};

    use super::{P2pNetworkKadEntry, P2pNetworkKadKey, P2pNetworkKadRoutingTable};

    const THIS_KEY: P2pNetworkKadKey = P2pNetworkKadKey(U256::ZERO);

    fn this_key() -> P2pNetworkKadKey {
        THIS_KEY.clone()
    }

    fn key_pow_2(pow: usize) -> P2pNetworkKadKey {
        P2pNetworkKadKey(U256::ONE.shl_vartime(pow))
    }

    fn key_rand() -> P2pNetworkKadKey {
        P2pNetworkKadKey(U256::random(&mut rand::thread_rng()))
    }

    fn peer_id_rand() -> PeerId {
        crate::identity::SecretKey::rand().public_key().peer_id()
    }

    fn entry(key: P2pNetworkKadKey) -> P2pNetworkKadEntry {
        let peer_id = peer_id_rand();
        P2pNetworkKadEntry {
            key,
            peer_id,
            addrs: vec![multiaddr!(Ip4([0; 4]), Tcp(1000_u16))],
            connection: super::ConnectionType::Connected,
        }
    }

    fn entry_with_peer_id(peer_id: PeerId) -> P2pNetworkKadEntry {
        let key = peer_id.try_into().expect("Error converting PeerId");
        P2pNetworkKadEntry {
            key,
            peer_id,
            addrs: vec![multiaddr!(Ip4([0; 4]), Tcp(1000_u16))],
            connection: super::ConnectionType::Connected,
        }
    }

    #[test]
    fn test_key_generation() {
        let random_peer_id = SecretKey::rand().public_key().peer_id();
        let libp2p_peer_id =
            libp2p_identity::PeerId::try_from(random_peer_id).expect("Conversion failed");
        let cid = CID::from(libp2p_peer_id);

        let key0 = P2pNetworkKadKey::try_from(&random_peer_id).expect("Conversion failed");
        let key1 = P2pNetworkKadKey::try_from(random_peer_id).expect("Conversion failed");
        let key2 = P2pNetworkKadKey::from(cid);

        assert_eq!(key0, key1);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_256_keys() {
        let mut rt: P2pNetworkKadRoutingTable = P2pNetworkKadRoutingTable::new(entry(this_key()));

        for i in 0..255 {
            println!("=== adding {i}...");
            let key = key_pow_2(255 - i);
            let entry = entry(key);
            let _ = rt.insert(entry);
            rt.assert_k_buckets();
        }
        println!("routing table: {rt:+#?}");
    }

    #[test]
    fn test_256_keys_rev() {
        let mut rt: P2pNetworkKadRoutingTable = P2pNetworkKadRoutingTable::new(entry(this_key()));

        for i in 0..255 {
            println!("=== adding {i}...");
            let key = key_pow_2(i);
            let entry = entry(key);
            let _ = rt.insert(entry);
            rt.assert_k_buckets();
        }
        println!("routing table: {rt:+#?}");
    }

    #[test]
    fn test_rand_keys() {
        let mut rt: P2pNetworkKadRoutingTable = P2pNetworkKadRoutingTable::new(entry(this_key()));
        for _ in 0..(256 * 256) {
            let key = key_rand();
            let entry = entry(key);
            let _ = rt.insert(entry);
            rt.assert_k_buckets();
        }
        println!("routing table: {rt:+#?}");
    }

    #[test]
    fn test_rand_peers_rand_this() {
        let mut rt: P2pNetworkKadRoutingTable =
            P2pNetworkKadRoutingTable::new(entry_with_peer_id(peer_id_rand()));
        for _ in 0..(256 * 256) {
            let peer_id = peer_id_rand();
            let entry = entry_with_peer_id(peer_id);
            let _ = rt.insert(entry);
            rt.assert_k_buckets();
        }
        println!("routing table: {rt:+#?}");
    }

    #[test]
    fn test_rand_peers() {
        let mut rt: P2pNetworkKadRoutingTable = P2pNetworkKadRoutingTable::new(entry(this_key()));
        for _ in 0..(256 * 256) {
            let peer_id = peer_id_rand();
            let entry = entry_with_peer_id(peer_id);
            let _ = rt.insert(entry);
            rt.assert_k_buckets();
        }
        println!("routing table: {rt:+#?}");
    }

    #[test]
    fn test_find_node_zero() {
        let this_entry = entry_with_peer_id(peer_id_rand());
        let mut rt: P2pNetworkKadRoutingTable = P2pNetworkKadRoutingTable::new(this_entry.clone());
        for _ in 0..(256 * 32) {
            let peer_id = peer_id_rand();
            let entry = entry_with_peer_id(peer_id);
            let _ = rt.insert(entry);
            rt.assert_k_buckets();
        }

        let entry = entry(this_key());
        let iter = rt.find_node(&entry.key);

        // let dist = entry.dist(&this_entry);
        // let index = dist.to_index().min(rt.buckets.len());

        // let iter = rt.buckets[index..]
        //     .iter()
        //     .flat_map(|bucket| bucket.iter())
        //     .filter(|e| &e.key != &entry.key)
        //     .take(20);
        let closest = BTreeSet::from_iter(iter);
        println!("{}", closest.len());

        let max_closest_dist = closest
            .iter()
            .max_by_key(|e| entry.dist(e))
            .expect("Failed to find entry");
        let min_non_closest_dist = rt
            .buckets
            .iter()
            .flat_map(|e| e.iter())
            .filter(|e| !closest.contains(*e))
            .min_by_key(|e| entry.dist(e))
            .expect("Failed to find entry");

        let max = entry.dist(max_closest_dist);
        let min = entry.dist(min_non_closest_dist);
        println!(
            "farthest {:#?} is closer than the closest {:#?}",
            max_closest_dist.key, min_non_closest_dist.key
        );
        assert!(min > max);
    }

    #[test]
    fn test_find_node_rand() {
        let mut rt: P2pNetworkKadRoutingTable = P2pNetworkKadRoutingTable::new(entry(this_key()));
        for _ in 0..(256 * 32) {
            let peer_id = peer_id_rand();
            let entry = entry_with_peer_id(peer_id);
            let _ = rt.insert(entry);
            rt.assert_k_buckets();
        }

        for _ in 0..(1024 * 16) {
            let peer_id = peer_id_rand();
            let entry = entry_with_peer_id(peer_id);

            let find_node = rt.find_node(&entry.key);
            let closest = BTreeSet::from_iter(find_node);

            let max_closest_dist = closest
                .iter()
                .max_by_key(|e| entry.dist(e))
                .expect("Error finding entry");
            let min_non_closest_dist = rt
                .buckets
                .iter()
                .flat_map(|e| e.iter())
                .filter(|e| e.key != entry.key && e.key != rt.this_key)
                .filter(|e| !closest.contains(*e))
                .min_by_key(|e| entry.dist(e))
                .expect("Error finding entry");

            let max = entry.dist(max_closest_dist);
            let min = entry.dist(min_non_closest_dist);
            if max > min {
                println!(
                    "farthest {:#?} should be closer than the closest {:#?}",
                    max_closest_dist.key, min_non_closest_dist.key
                );
                panic!("min is {min:#?}\nmax is {max:#?}");
            }
        }
    }

    /// Tests that `find_node` returns entries in order of increasing distance.
    #[test]
    fn test_closest_peers_rand() {
        let mut rt: P2pNetworkKadRoutingTable = P2pNetworkKadRoutingTable::new(entry(this_key()));
        for _ in 0..(256 * 32) {
            let peer_id = peer_id_rand();
            let entry = entry_with_peer_id(peer_id);
            let _ = rt.insert(entry);
            rt.assert_k_buckets();
        }

        for _ in 0..(16 * 1024) {
            let peer_id = peer_id_rand();
            let entry = entry_with_peer_id(peer_id);

            let mut prev = None;
            for e in rt.find_node(&entry.key) {
                let dist = entry.dist(e);
                if let Some((prev, prev_dist)) = prev {
                    if dist <= prev_dist {
                        panic!(
                            "incorrect order\n{:#?}\nshould go before\n{:#?}",
                            e.key, prev
                        );
                    }
                }
                prev = Some((e.clone(), dist));
            }
        }
    }
}

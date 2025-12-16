use std::{cmp::Ord, collections::BTreeMap, ops::RangeBounds};

use crate::bug_condition;

#[derive(Clone)]
pub struct DistributedPool<State, Key: Ord> {
    counter: u64,
    list: BTreeMap<u64, State>,
    by_key: BTreeMap<Key, u64>,
}

impl<State, Key: Ord> Default for DistributedPool<State, Key> {
    fn default() -> Self {
        Self {
            counter: 0,
            list: Default::default(),
            by_key: Default::default(),
        }
    }
}

impl<State, Key> DistributedPool<State, Key>
where
    State: AsRef<Key>,
    Key: Ord + Clone,
{
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.by_key
            .get(key)
            .is_some_and(|i| self.list.contains_key(i))
    }

    pub fn get(&self, key: &Key) -> Option<&State> {
        self.by_key.get(key).and_then(|i| self.list.get(i))
    }

    fn get_mut(&mut self, key: &Key) -> Option<&mut State> {
        self.by_key.get(key).and_then(|i| self.list.get_mut(i))
    }

    pub fn range<R>(&self, range: R) -> impl '_ + DoubleEndedIterator<Item = (u64, &'_ State)>
    where
        R: RangeBounds<u64>,
    {
        self.list.range(range).map(|(k, v)| (*k, v))
    }

    pub fn last_index(&self) -> u64 {
        self.list.last_key_value().map_or(0, |(k, _)| *k)
    }

    pub fn insert(&mut self, state: State) {
        let key = state.as_ref().clone();
        self.list.insert(self.counter, state);
        self.by_key.insert(key, self.counter);
        self.counter = self.counter.saturating_add(1);
    }

    pub fn update<F, R>(&mut self, key: &Key, f: F) -> Option<R>
    where
        F: FnOnce(&mut State) -> R,
    {
        let mut state = self.remove(key)?;
        let res = f(&mut state);
        self.insert(state);
        Some(res)
    }

    /// Don't use if the change needs to be synced with other peers.
    pub fn silent_update<F, R>(&mut self, key: &Key, f: F) -> Option<R>
    where
        F: FnOnce(&mut State) -> R,
    {
        self.get_mut(key).map(f)
    }

    pub fn remove(&mut self, key: &Key) -> Option<State> {
        let index = self.by_key.remove(key)?;
        self.list.remove(&index)
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Key, &State) -> bool,
    {
        self.retain_and_update(|key, state| f(key, state))
    }

    pub fn retain_and_update<F>(&mut self, mut f: F)
    where
        F: FnMut(&Key, &mut State) -> bool,
    {
        let list = &mut self.list;
        self.by_key.retain(|key, index| {
            let Some(v) = list.get_mut(index) else {
                bug_condition!("Pool: key found in the index, but the item not found");
                return false;
            };
            if f(key, v) {
                return true;
            }
            list.remove(index);
            false
        });
    }

    pub fn states(&self) -> impl Iterator<Item = &State> {
        self.list.values()
    }
}

impl<State, Key> DistributedPool<State, Key>
where
    State: AsRef<Key>,
    Key: Ord + Clone,
{
    pub fn next_messages_to_send<F, T>(
        &self,
        (index, limit): (u64, u8),
        extract_message: F,
    ) -> (Vec<T>, u64, u64)
    where
        F: Fn(&State) -> Option<T>,
    {
        if limit == 0 {
            let index = index.saturating_sub(1);
            return (vec![], index, index);
        }

        self.range(index..)
            .try_fold(
                (vec![], None),
                |(mut list, mut first_index), (index, job)| {
                    if let Some(data) = extract_message(job) {
                        let first_index = *first_index.get_or_insert(index);
                        list.push(data);
                        if list.len() >= limit as usize {
                            return Err((list, first_index, index));
                        }
                    }

                    Ok((list, first_index))
                },
            )
            // Loop iterated on whole list.
            .map(|(list, first_index)| (list, first_index.unwrap_or(index), self.last_index()))
            // Loop preemptively ended.
            .unwrap_or_else(|v| v)
    }
}

mod ser {
    use super::*;
    use serde::{ser::SerializeStruct, Deserialize, Serialize};

    #[derive(Deserialize)]
    struct Pool<State> {
        counter: u64,
        list: BTreeMap<u64, State>,
    }

    impl<State, Key> Serialize for super::DistributedPool<State, Key>
    where
        State: Serialize,
        Key: Ord,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut s = serializer.serialize_struct("Pool", 2)?;
            s.serialize_field("counter", &self.counter)?;
            s.serialize_field("list", &self.list)?;
            s.end()
        }
    }
    impl<'de, State, Key> Deserialize<'de> for super::DistributedPool<State, Key>
    where
        State: Deserialize<'de> + AsRef<Key>,
        Key: Ord + Clone,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let v = Pool::<State>::deserialize(deserializer)?;
            let by_key = v
                .list
                .iter()
                .map(|(k, v)| (v.as_ref().clone(), *k))
                .collect();
            Ok(Self {
                counter: v.counter,
                list: v.list,
                by_key,
            })
        }
    }
}

impl<State, Key: Ord> std::fmt::Debug for DistributedPool<State, Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pool")
            .field("counter", &self.counter)
            .field("len", &self.list.len())
            .finish()
    }
}

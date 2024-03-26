use std::fmt;
use std::marker::PhantomData;

pub trait RequestIdType {
    fn request_id_type() -> &'static str;
}

#[cfg_attr(feature = "fuzzing", derive(fuzzcheck::DefaultMutator))]
pub struct RequestId<T> {
    locator: usize,
    counter: usize,
    _phantom_request_type: PhantomData<T>,
}

impl<T> RequestId<T> {
    pub(super) fn new(locator: usize, counter: usize) -> Self {
        Self {
            locator,
            counter,
            _phantom_request_type: Default::default(),
        }
    }

    pub fn new_unchecked(locator: usize, counter: usize) -> Self {
        Self {
            locator,
            counter,
            _phantom_request_type: Default::default(),
        }
    }

    pub fn locator(&self) -> usize {
        self.locator
    }

    pub fn counter(&self) -> usize {
        self.counter
    }
}

mod serde_impl {
    use serde::{Deserialize, Serialize};

    use super::RequestIdType;

    #[derive(Serialize, Deserialize)]
    struct RequestId {
        locator: usize,
        counter: usize,
    }

    impl<T: RequestIdType> Serialize for super::RequestId<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            if serializer.is_human_readable() {
                return Serialize::serialize(&self.to_string(), serializer);
            }
            let id = RequestId {
                locator: self.locator,
                counter: self.counter,
            };
            Serialize::serialize(&id, serializer)
        }
    }

    impl<'de, T: RequestIdType> Deserialize<'de> for super::RequestId<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                let s: &str = Deserialize::deserialize(deserializer)?;
                return s.parse().or(Err(serde::de::Error::custom("invalid id")));
            }
            let id = RequestId::deserialize(deserializer)?;
            Ok(Self {
                locator: id.locator,
                counter: id.counter,
                _phantom_request_type: Default::default(),
            })
        }
    }
}

impl<T> Eq for RequestId<T> {}
impl<T> PartialEq for RequestId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<T> Ord for RequestId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.counter
            .cmp(&other.counter)
            .then(self.locator.cmp(&other.locator))
    }
}
impl<T> PartialOrd for RequestId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> std::hash::Hash for RequestId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.counter.hash(state);
        self.locator.hash(state);
    }
}

impl<T> std::str::FromStr for RequestId<T> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (locator, counter) = s.split_once('_').ok_or(())?;
        Ok(Self {
            locator: locator.parse().or(Err(()))?,
            counter: counter.parse().or(Err(()))?,
            _phantom_request_type: Default::default(),
        })
    }
}

impl<T> fmt::Display for RequestId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.locator, self.counter)
    }
}

impl<T: RequestIdType> fmt::Debug for RequestId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", T::request_id_type(), self)
    }
}

impl<T> Clone for RequestId<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Copy for RequestId<T> {}

use std::fmt;
use std::marker::PhantomData;

pub trait RequestIdType {
    fn request_id_type() -> &'static str;
}

#[cfg_attr(feature = "fuzzing", derive(fuzzcheck::DefaultMutator))]
#[derive(Hash, Ord, PartialOrd, Eq, PartialEq)]
// TODO(binier): impl manually all above traits so that they don't add
// bounds for `T` as it's only used as `PhantomData<T>`.
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
            let id = RequestId::deserialize(deserializer)?;
            Ok(Self {
                locator: id.locator,
                counter: id.counter,
                _phantom_request_type: Default::default(),
            })
        }
    }
}

impl<T: RequestIdType> fmt::Display for RequestId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({}:{})",
            T::request_id_type(),
            self.locator,
            self.counter
        )
    }
}

impl<T: RequestIdType> fmt::Debug for RequestId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(T::request_id_type())
            .field("locator", &self.locator)
            .field("counter", &self.counter)
            .finish()
    }
}

impl<T> Clone for RequestId<T> {
    fn clone(&self) -> Self {
        Self {
            locator: self.locator,
            counter: self.counter,
            _phantom_request_type: self._phantom_request_type.clone(),
        }
    }
}

impl<T> Copy for RequestId<T> {}

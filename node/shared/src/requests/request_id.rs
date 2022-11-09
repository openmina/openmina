use std::fmt;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

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

impl<T: RequestIdType> Serialize for RequestId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct(T::request_id_type(), 2)?;
        s.serialize_field("locator", &self.locator)?;
        s.serialize_field("counter", &self.counter)?;
        s.end()
    }
}

impl<'de, T: RequestIdType> Deserialize<'de> for RequestId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RequestIdVisitor<T> {
            _phantom_t: PhantomData<T>,
        }
        impl<'de, T> serde::de::Visitor<'de> for RequestIdVisitor<T> {
            type Value = RequestId<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a `RequestId<T>` with 2 fields: `locator` and `counter`"
                )
            }
        }
        let visitor = RequestIdVisitor {
            _phantom_t: Default::default(),
        };
        deserializer.deserialize_struct(T::request_id_type(), &["locator", "counter"], visitor)
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

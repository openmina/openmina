// TODO:
use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use salsa20::{cipher::generic_array::GenericArray, cipher::KeyIvInit, XSalsa20};

#[derive(Clone)]
pub struct XSalsa20Wrapper {
    inner: XSalsa20,
}

impl XSalsa20Wrapper {
    pub fn new(shared_secret: &[u8; 32], nonce: &[u8; 24]) -> Self {
        XSalsa20Wrapper {
            inner: XSalsa20::new(
                GenericArray::from_slice(shared_secret),
                GenericArray::from_slice(nonce),
            ),
        }
    }
}

impl Deref for XSalsa20Wrapper {
    type Target = XSalsa20;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for XSalsa20Wrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl fmt::Debug for XSalsa20Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XSalsa20").finish()
    }
}

impl serde::Serialize for XSalsa20Wrapper {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unimplemented!()
    }
}

impl<'de> serde::Deserialize<'de> for XSalsa20Wrapper {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

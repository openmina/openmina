use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use serde::{de::Deserializer, Serializer};

pub fn ark_serialize<S, A: CanonicalSerialize>(a: &A, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut bytes = vec![];
    a.serialize(&mut bytes).map_err(serde::ser::Error::custom)?;
    s.serialize_bytes(&bytes)
}

pub fn ark_deserialize<'de, D, A: CanonicalDeserialize>(data: D) -> Result<A, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<u8> = serde::de::Deserialize::deserialize(data)?;
    let a = A::deserialize(s.as_slice());
    a.map_err(serde::de::Error::custom)
}

use serde::{Deserialize, Serialize};
use slab::Slab;
use std::fmt;

mod request_id;
pub use request_id::{RequestId, RequestIdType};

mod rpc_id;
pub use rpc_id::{RpcId, RpcIdType};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PendingRequest<Request> {
    counter: usize,
    request: Request,
}

pub struct PendingRequests<IdType: RequestIdType, Request> {
    list: Slab<PendingRequest<Request>>,
    counter: usize,
    last_added_req_id: RequestId<IdType>,
}

impl<IdType, Request> PendingRequests<IdType, Request>
where
    IdType: RequestIdType,
{
    pub fn new() -> Self {
        Self {
            list: Slab::new(),
            counter: 0,
            last_added_req_id: RequestId::new(0, 0),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn counter(&self) -> usize {
        self.counter
    }

    #[inline]
    pub fn last_added_req_id(&self) -> RequestId<IdType> {
        self.last_added_req_id
    }

    #[inline]
    pub fn next_req_id(&self) -> RequestId<IdType> {
        RequestId::new(self.list.vacant_key(), self.counter.wrapping_add(1))
    }

    #[inline]
    pub fn contains(&self, id: RequestId<IdType>) -> bool {
        self.get(id).is_some()
    }

    #[inline]
    pub fn get(&self, id: RequestId<IdType>) -> Option<&Request> {
        self.list
            .get(id.locator())
            .filter(|req| req.counter == id.counter())
            .map(|x| &x.request)
    }

    #[inline]
    pub fn get_mut(&mut self, id: RequestId<IdType>) -> Option<&mut Request> {
        self.list
            .get_mut(id.locator())
            .filter(|req| req.counter == id.counter())
            .map(|x| &mut x.request)
    }

    #[inline]
    pub fn add(&mut self, request: Request) -> RequestId<IdType> {
        self.counter = self.counter.wrapping_add(1);

        let locator = self.list.insert(PendingRequest {
            counter: self.counter,
            request,
        });

        let req_id = RequestId::new(locator, self.counter);
        self.last_added_req_id = req_id;

        req_id
    }

    #[inline]
    pub fn remove(&mut self, id: RequestId<IdType>) -> Option<Request> {
        self.get(id)?;
        let removed_req = self.list.remove(id.locator()).request;
        Some(removed_req)
    }

    pub fn iter(&self) -> impl Iterator<Item = (RequestId<IdType>, &Request)> {
        self.list
            .iter()
            .map(|(locator, req)| (RequestId::new(locator, req.counter), &req.request))
    }
}

impl<IdType, Request> Default for PendingRequests<IdType, Request>
where
    IdType: RequestIdType,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<IdType, Request> Serialize for PendingRequests<IdType, Request>
where
    IdType: RequestIdType,
    Request: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PendingRequests", 2)?;
        s.serialize_field("list", &self.list)?;
        s.serialize_field("counter", &self.counter)?;
        s.serialize_field("last_added_req_id", &self.last_added_req_id)?;
        s.end()
    }
}

impl<'de, IdType, Request> Deserialize<'de> for PendingRequests<IdType, Request>
where
    IdType: RequestIdType,
    Request: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        struct RequestsVisitor<IdType, Request>(std::marker::PhantomData<(IdType, Request)>);

        const FIELDS: &'static [&'static str] = &["list", "counter", "last_added_req_id"];

        impl<'de, IdType, Request> Visitor<'de> for RequestsVisitor<IdType, Request>
        where
            IdType: RequestIdType,
            Request: Deserialize<'de>,
        {
            type Value = PendingRequests<IdType, Request>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct PendingRequests")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<PendingRequests<IdType, Request>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                Ok(PendingRequests {
                    list: seq
                        .next_element()?
                        .ok_or_else(|| de::Error::missing_field("list"))?,
                    counter: seq
                        .next_element()?
                        .ok_or_else(|| de::Error::missing_field("counter"))?,
                    last_added_req_id: seq
                        .next_element()?
                        .ok_or_else(|| de::Error::missing_field("last_added_req_id"))?,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<PendingRequests<IdType, Request>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut list = None;
                let mut counter = None;
                let mut last_added_req_id = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "list" => {
                            if list.is_some() {
                                return Err(de::Error::duplicate_field("list"));
                            }
                            list = Some(map.next_value()?);
                        }
                        "counter" => {
                            if counter.is_some() {
                                return Err(de::Error::duplicate_field("counter"));
                            }
                            counter = Some(map.next_value()?);
                        }
                        "last_added_req_id" => {
                            if last_added_req_id.is_some() {
                                return Err(de::Error::duplicate_field("last_added_req_id"));
                            }
                            last_added_req_id = Some(map.next_value()?);
                        }
                        field => return Err(de::Error::unknown_field(field, FIELDS)),
                    }
                }
                let list = list.ok_or_else(|| de::Error::missing_field("list"))?;
                let counter = counter.ok_or_else(|| de::Error::missing_field("counter"))?;
                let last_added_req_id = last_added_req_id
                    .ok_or_else(|| de::Error::missing_field("last_added_req_id"))?;
                Ok(PendingRequests {
                    list,
                    counter,
                    last_added_req_id,
                })
            }
        }

        let visitor = RequestsVisitor(Default::default());
        deserializer.deserialize_struct("PendingRequests", FIELDS, visitor)
    }
}

impl<IdType, Request> fmt::Debug for PendingRequests<IdType, Request>
where
    IdType: RequestIdType,
    Request: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingRequests")
            .field("list", &self.list)
            .field("counter", &self.counter)
            .field("last_added_req_id", &self.last_added_req_id)
            .finish()
    }
}

impl<IdType, Request> Clone for PendingRequests<IdType, Request>
where
    IdType: RequestIdType,
    Request: Clone,
{
    fn clone(&self) -> Self {
        Self {
            list: self.list.clone(),
            counter: self.counter,
            last_added_req_id: self.last_added_req_id,
        }
    }
}

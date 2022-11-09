use serde::{Deserialize, Serialize};
use slab::Slab;

mod request_id;
pub use request_id::{RequestId, RequestIdType};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PendingRequest<Request> {
    counter: usize,
    request: Request,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

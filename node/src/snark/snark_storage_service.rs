use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::Path,
    sync::Arc,
    thread,
};

use ledger::scan_state::currency::Fee;
use openmina_core::{channels::mpsc, snark::SnarkJobId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(SnarkJobId);

#[derive(Debug, Clone)]
pub struct Snark(Arc<openmina_core::snark::Snark>);

impl PartialEq for Snark {
    fn eq(&self, other: &Self) -> bool {
        self.0.job_id() == other.0.job_id()
    }
}

impl Eq for Snark {}

impl PartialOrd for Snark {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.job_id().partial_cmp(&other.0.job_id())
    }
}

impl Ord for Snark {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.job_id().cmp(&other.0.job_id())
    }
}

impl Snark {
    fn address(&self) -> Address {
        Address(self.0.job_id())
    }
}

struct SnarkStorage {
    snarks: BTreeMap<Address, Snark>,
    snarks_by_fee: BTreeMap<Fee, BTreeSet<Snark>>,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

fn map_set_insert(map: &mut BTreeMap<Fee, BTreeSet<Snark>>, snark: Snark) {
    let fee = (&snark.0.fee).into();
    match map.get_mut(&fee) {
        Some(set) => {
            set.replace(snark);
        }
        None => {
            let mut set = BTreeSet::new();
            set.replace(snark);
            map.insert(fee, set);
        }
    }
}

impl SnarkStorage {
    fn new() -> io::Result<Self> {
        Ok(Self {
            snarks: BTreeMap::new(),
            snarks_by_fee: BTreeMap::new(),
        })
    }

    fn insert(&mut self, snark: openmina_core::snark::Snark) -> Result<Address> {
        let item = Snark(Arc::new(snark));
        let addr = item.address();
        // ignore discarded duplicates; the SNARK pool always chooses the best one,
        // so if we get a duplicate here, we always want to replace the old one with
        // the new.
        self.snarks.insert(addr.clone(), item.clone());
        map_set_insert(&mut self.snarks_by_fee, item);
        Ok(addr)
    }

    fn get(&self, addr: &Address) -> Result<Option<Snark>> {
        Ok(self.snarks.get(addr).map(|snark| snark.clone()))
    }

    fn drop(&mut self, addr: &Address) -> Result<()> {
        if let Some(snark) = self.snarks.remove(addr) {
            let fee = (&snark.0.fee).into();
            if let Some(set) = self.snarks_by_fee.get_mut(&fee) {
                set.remove(&snark);
            } else {
                openmina_core::warn!(
                    openmina_core::log::system_time();
                    kind = "SNARK removal",
                    message = "SNARK not found in fee index",
                    fee = fee.as_u64(),
                    addr = addr.0.to_string(),
                )
            }
        } else {
            openmina_core::warn!(
                openmina_core::log::system_time();
                kind = "SNARK removal",
                message = "SNARK not found",
                addr = addr.0.to_string(),
            )
        }
        Ok(())
    }

    fn response<T, F>(constr: F, result: std::result::Result<T, StorageError>) -> SnarkResponse
        where F: FnOnce(T) -> SnarkResponse
    {        
        match result {
            Ok(t) => constr(t),
            Err(e) => SnarkResponse::Error(e),
        }
    }

    fn handle_request(&mut self, request: SnarkRequest) -> SnarkResponse {
        match request {
            SnarkRequest::Store(snark) => Self::response(SnarkResponse::Stored, self.insert(snark)),
            SnarkRequest::Get(addr) => Self::response(SnarkResponse::Retrieved, self.get(&addr)),
            SnarkRequest::Drop(addr) => Self::response(|()| SnarkResponse::Dropped, self.drop(&addr)),
        }
    }
}

enum SnarkRequest {
    Store(openmina_core::snark::Snark),
    Get(Address),
    Drop(Address),
}

enum SnarkResponse {
    Stored(Address),
    Retrieved(Option<Snark>),
    Dropped,
    Error(StorageError)
}

struct SnarkRequestWithChan {
    request: SnarkRequest,
    responder: Option<std::sync::mpsc::SyncSender<SnarkResponse>>,
}

pub(super) struct SnarkCaller(mpsc::UnboundedSender<SnarkRequestWithChan>);

impl SnarkCaller {
    fn call(&self, request: SnarkRequest) {
        self.0
            .send(SnarkRequestWithChan {
                request,
                responder: None,
            })
            .unwrap();
    }

    fn call_sync(
        &self,
        request: SnarkRequest,
    ) -> std::result::Result<SnarkResponse, std::sync::mpsc::RecvError> {
        let (responder, receiver) = std::sync::mpsc::sync_channel(0);
        self.0
            .send(SnarkRequestWithChan {
                request,
                responder: Some(responder),
            })
            .unwrap();
        receiver.recv()
    }
}

pub struct SnarkStorageService {
    caller: SnarkCaller,
    join_handle: thread::JoinHandle<()>,
}

impl SnarkStorageService {
    pub fn spawn(_location: &Path) -> Result<Self> {
        let mut storage = SnarkStorage::new()?;
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let caller = SnarkCaller(sender);
        let join_handle = thread::spawn(move || {
            while let Some(SnarkRequestWithChan { request, responder }) = receiver.blocking_recv() {
                let response = storage.handle_request(request);
                responder.map(|r| r.send(response).unwrap());
            }
        });
        Ok(Self {
            caller,
            join_handle,
        })
    }

    pub fn store(&self, snark: openmina_core::snark::Snark) -> Result<Address> {
        match self.caller.call_sync(SnarkRequest::Store(snark)).unwrap() {
            SnarkResponse::Stored(addr) => Ok(addr),
            SnarkResponse::Error(e) => Err(e),
            _ => unreachable!(),
        }
    }

    pub fn get(&self, addr: Address) -> Result<Option<Snark>> {
        match self.caller.call_sync(SnarkRequest::Get(addr)).unwrap() {
            SnarkResponse::Retrieved(snark) => Ok(snark),
            SnarkResponse::Error(e) => Err(e),
            _ => unreachable!(),
        }
    }

    pub fn drop(&self, addr: Address) -> Result<()> {
        match self.caller.call_sync(SnarkRequest::Drop(addr)).unwrap() {
            SnarkResponse::Dropped => Ok(()),
            SnarkResponse::Error(e) => Err(e),
            _ => unreachable!(),
        }
    }

    pub fn close(self) -> thread::JoinHandle<()> {
        self.join_handle
    }
}

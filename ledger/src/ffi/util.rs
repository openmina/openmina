use std::{collections::HashSet, hash::Hash, io::Cursor};

use binprot::{BinProtRead, BinProtWrite};
use mina_hasher::Fp;
use ocaml_interop::*;

use crate::{Account, AccountIndex, Address, BigInt};

pub fn deserialize<T: BinProtRead>(bytes: &[u8]) -> T {
    let mut cursor = Cursor::new(bytes);
    T::binprot_read(&mut cursor).unwrap()
}

pub fn serialize<T: BinProtWrite>(obj: &T) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(10000); // TODO: fix this
    obj.binprot_write(&mut bytes).unwrap();
    bytes
}

pub fn get_list_of<'a, T>(
    rt: &'a mut &mut OCamlRuntime,
    list: OCamlRef<OCamlList<OCamlBytes>>,
) -> Vec<T>
where
    T: BinProtRead,
{
    let mut list_ref = rt.get(list);
    let mut list = Vec::with_capacity(2048);

    while let Some((head, tail)) = list_ref.uncons() {
        let object: T = deserialize(head.as_bytes());
        list.push(object);
        list_ref = tail;
    }

    list
}

pub fn get_set_of<'a, T>(
    rt: &'a mut &mut OCamlRuntime,
    list: OCamlRef<OCamlList<OCamlBytes>>,
) -> HashSet<T>
where
    T: BinProtRead + Hash + Eq,
{
    let mut list_ref = rt.get(list);
    let mut set = HashSet::with_capacity(2048);

    while let Some((head, tail)) = list_ref.uncons() {
        let object: T = deserialize(head.as_bytes());
        set.insert(object);
        list_ref = tail;
    }

    set
}

pub fn get_list_addr_account<'a>(
    rt: &'a mut &mut OCamlRuntime,
    list: OCamlRef<OCamlList<(String, OCamlBytes)>>,
) -> Vec<(Address, Account)> {
    let mut list_ref = rt.get(list);
    let mut list = Vec::with_capacity(2048);

    while let Some((head, tail)) = list_ref.uncons() {
        let addr = head.fst().as_str();
        let account = head.snd().as_bytes();

        let addr = Address::try_from(addr).unwrap();
        let object: Account = deserialize(account);
        list.push((addr, object));

        list_ref = tail;
    }

    list
}

pub fn get_addr(rt: &mut &mut OCamlRuntime, addr: OCamlRef<String>) -> Address {
    let addr_ref = rt.get(addr);
    Address::try_from(addr_ref.as_str()).unwrap()
}

pub fn get<'a, T>(rt: &'a mut &mut OCamlRuntime, object: OCamlRef<OCamlBytes>) -> T
where
    T: BinProtRead,
{
    let object_ref = rt.get(object);
    deserialize(object_ref.as_bytes())
}

pub fn get_index(rt: &mut &mut OCamlRuntime, index: OCamlRef<OCamlInt>) -> AccountIndex {
    let index: i64 = index.to_rust(rt);
    let index: u64 = index.try_into().unwrap();
    AccountIndex(index)
}

pub fn hash_to_ocaml(hash: Fp) -> Vec<u8> {
    let hash: BigInt = hash.into();
    serialize(&hash)
}

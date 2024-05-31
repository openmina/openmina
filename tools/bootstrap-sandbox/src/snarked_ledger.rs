use binprot::{BinProtRead, BinProtWrite};
use std::{future::Future, io, pin::Pin};
use thiserror::Error;

use ledger::{Account, AccountIndex, Address, BaseLedger, Database, Mask};
use mina_p2p_messages::{list::List, rpc::AnswerSyncLedgerQueryV2, v2};

use super::client::Client;

pub struct SnarkedLedger {
    pub inner: Mask,
    // NOTE: it is not the same as the merkle tree root
    pub top_hash: Option<v2::LedgerHash>,
    pub num: u32,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Io(#[from] io::Error),
}

impl SnarkedLedger {
    pub fn empty() -> Self {
        SnarkedLedger {
            inner: Mask::new_root(Database::create(35)),
            top_hash: None,
            num: 0,
        }
    }

    // for debugging
    pub fn store_bin<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: io::Write,
    {
        let accounts = self.inner.fold(vec![], |mut accounts, account| {
            accounts.push(account.clone());
            accounts
        });
        self.top_hash.binprot_write(&mut writer)?;
        accounts.binprot_write(&mut writer)
    }

    pub fn load_bin<R>(mut reader: R) -> Result<Self, binprot::Error>
    where
        R: io::Read,
    {
        let top_hash = Option::binprot_read(&mut reader)?;
        let accounts = List::<Account>::binprot_read(&mut reader)?;

        let num = accounts.len() as _;
        let mut inner = Mask::new_root(Database::create(35));
        for account in accounts {
            let account_id = account.id();
            inner.get_or_create_account(account_id, account).unwrap();
        }

        let _ = inner.merkle_root();

        Ok(SnarkedLedger {
            inner,
            top_hash,
            num,
        })
    }

    pub async fn sync_new(&mut self, client: &mut Client, root: &v2::LedgerHash) {
        let q = v2::MinaLedgerSyncLedgerQueryStableV1::NumAccounts;
        let r = match client
            .rpc::<AnswerSyncLedgerQueryV2>((root.0.clone(), q))
            .await
            .unwrap()
            .0
        {
            Ok(v) => v,
            Err(e) => panic!("answer_sync_ledger returned error: {e}"),
        };
        let (num, hash) = match r {
            v2::MinaLedgerSyncLedgerAnswerStableV2::NumAccounts(num, hash) => (num.0, hash),
            _ => panic!(),
        };
        self.top_hash = Some(hash.clone());
        self.num = num as _;

        if self.inner.num_accounts() > num as _ {
            self.inner = Mask::new_root(Database::create(35));
        }

        self.sync_at_depth_new(client, root.clone(), hash.clone(), 0, 0)
            .await;
        let actual_hash = self.inner.merkle_root();
        let actual_hash = v2::LedgerHash::from(v2::MinaBaseLedgerHash0StableV1(actual_hash.into()));
        assert_eq!(actual_hash, root.clone());
    }

    fn sync_at_depth_boxed_new<'a, 'b: 'a>(
        &'b mut self,
        client: &'a mut Client,
        root: v2::LedgerHash,
        hash: v2::LedgerHash,
        depth: i32,
        pos: u32,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(self.sync_at_depth_new(client, root, hash, depth, pos))
    }

    async fn sync_at_depth_new(
        &mut self,
        client: &mut Client,
        root: v2::LedgerHash,
        hash: v2::LedgerHash,
        depth: i32,
        pos: u32,
    ) {
        let addr = Address::from_index(AccountIndex(pos as _), depth as _);
        let actual_hash = self.inner.get_inner_hash_at_addr(addr.clone()).unwrap();
        if depth == 0 && root.0 == actual_hash.into() || depth > 0 && hash.0 == actual_hash.into() {
            return;
        }

        if depth == 32 {
            let p = pos.to_be_bytes().to_vec();
            let q = v2::MinaLedgerSyncLedgerQueryStableV1::WhatContents(
                v2::MerkleAddressBinableArgStableV1((depth as u64).into(), p.into()),
            );
            log::info!("{}", serde_json::to_string(&q).unwrap());
            let r = client
                .rpc::<AnswerSyncLedgerQueryV2>((root.0.clone(), q))
                .await
                .unwrap()
                .0;
            match r {
                Err(err) => {
                    log::error!("num: {}, error: {err}", self.num);
                }
                Ok(v2::MinaLedgerSyncLedgerAnswerStableV2::ContentsAre(accounts)) => {
                    for (o, account) in accounts.into_iter().enumerate() {
                        let account = Account::from(&account);
                        self.inner
                            .set_at_index(
                                AccountIndex((pos * 8) as u64 + o as u64),
                                Box::new(account),
                            )
                            .unwrap();
                    }
                }
                _ => panic!(),
            }
        } else {
            let b = ((depth as usize + 7) / 8).min(4);
            let p = if depth > 0 {
                pos * (1 << (32 - depth))
            } else {
                0
            };
            let p = p.to_be_bytes()[..b].to_vec();
            let q = v2::MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(
                v2::MerkleAddressBinableArgStableV1((depth as u64).into(), p.into()),
            );
            log::info!("{}", serde_json::to_string(&q).unwrap());
            let r = client
                .rpc::<AnswerSyncLedgerQueryV2>((root.0.clone(), q))
                .await
                .unwrap()
                .0
                .unwrap();
            match r {
                v2::MinaLedgerSyncLedgerAnswerStableV2::ChildHashesAre(l, r) => {
                    self.sync_at_depth_boxed_new(client, root.clone(), l, depth + 1, pos * 2)
                        .await;
                    self.sync_at_depth_boxed_new(client, root.clone(), r, depth + 1, pos * 2 + 1)
                        .await;
                }
                _ => panic!(),
            };
        }

        let addr = Address::from_index(AccountIndex(pos as _), depth as _);
        let actual_hash = self.inner.get_inner_hash_at_addr(addr).unwrap();
        let actual_hash = v2::LedgerHash::from(v2::MinaBaseLedgerHash0StableV1(actual_hash.into()));
        if depth == 0 {
            assert_eq!(root, actual_hash);
        } else {
            assert_eq!(hash, actual_hash);
        }
    }

    pub fn serve_query(
        &mut self,
        q: v2::MinaLedgerSyncLedgerQueryStableV1,
    ) -> v2::MinaLedgerSyncLedgerAnswerStableV2 {
        log::info!("query: {q:?}");
        match q {
            v2::MinaLedgerSyncLedgerQueryStableV1::NumAccounts => {
                v2::MinaLedgerSyncLedgerAnswerStableV2::NumAccounts(
                    (self.num as u64).into(),
                    self.top_hash.as_ref().unwrap().clone(),
                )
            }
            v2::MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(address) => {
                let addr = Address::from(address);

                let hash = self
                    .inner
                    .get_inner_hash_at_addr(addr.child_left())
                    .unwrap();
                let left = v2::LedgerHash::from(v2::MinaBaseLedgerHash0StableV1(hash.into()));

                let hash = self
                    .inner
                    .get_inner_hash_at_addr(addr.child_right())
                    .unwrap();
                let right = v2::LedgerHash::from(v2::MinaBaseLedgerHash0StableV1(hash.into()));

                v2::MinaLedgerSyncLedgerAnswerStableV2::ChildHashesAre(left, right)
            }
            v2::MinaLedgerSyncLedgerQueryStableV1::WhatContents(address) => {
                let addr = Address::from(address);

                let depth = addr.length();
                let pos = addr.to_index().0;

                let mut accounts = List::new();
                let mut offset = 0;
                let batch_length = 1u64 << (35 - depth);
                loop {
                    if offset == batch_length {
                        break;
                    }

                    let pos = pos * batch_length + offset;
                    offset += 1;
                    if pos == self.num as u64 {
                        break;
                    }
                    let addr = Address::from_index(AccountIndex(pos as _), 35);
                    let account = self.inner.get(addr);
                    if let Some(account) = account {
                        accounts.push_back((&*account).into());
                    } else {
                        break;
                    }
                }
                v2::MinaLedgerSyncLedgerAnswerStableV2::ContentsAre(accounts)
            }
        }
    }
}

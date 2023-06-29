use ledger::{AccountIndex, BaseLedger, HashesMatrix, Mask};
use mina_hasher::Fp;
use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};

use super::{ledger_empty_hash_at_depth, LedgerAddress, LedgerId, LEDGER_DEPTH};

fn ledger_hash(depth: usize, left: Fp, right: Fp) -> Fp {
    let height = LEDGER_DEPTH - depth - 1;
    let param = format!("MinaMklTree{:03}", height);
    ledger::hash_with_kimchi(param.as_str(), &[left, right])
}

pub trait LedgerService: redux::Service {
    fn hashes_matrix(&mut self, id: &LedgerId) -> &mut HashesMatrix;
    fn get_ledger(&mut self, id: &LedgerId) -> &mut Mask;

    fn hashes_matrix_set(
        &mut self,
        id: &LedgerId,
        parent: &LedgerAddress,
        (left, right): (LedgerHash, LedgerHash),
    ) -> Result<(), ()> {
        let matrix = self.hashes_matrix(id);
        let (left, right) = (left.0.to_field(), right.0.to_field());
        let hash = ledger_hash(parent.length(), left, right);

        if &hash != matrix.get(parent).ok_or(())? {
            return Err(());
        }

        matrix.set(&parent.child_left(), left);
        matrix.set(&parent.child_right(), right);

        Ok(())
    }

    fn accounts_set(
        &mut self,
        id: &LedgerId,
        parent: &LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ) -> Result<(), ()> {
        let mask = self.get_ledger(id);
        // TODO(binier): validate hashes
        let mut addr = parent.clone();
        let first_addr = loop {
            if addr.length() == LEDGER_DEPTH {
                break addr;
            }
            addr = addr.child_left();
        };
        let first_index = first_addr.to_index();
        accounts
            .into_iter()
            .enumerate()
            .try_for_each(|(index, account)| {
                let index = AccountIndex(first_index.0 + index as u64);
                mask.set_at_index(index, account.into())
            })?;

        Ok(())
    }

    fn assert_downloaded_hashes(&mut self, id: &LedgerId) {
        let calculated_root = self.get_ledger(id).merkle_root();
        assert_eq!(id.hash.0.to_fp().unwrap(), calculated_root);

        let mut addr = LedgerAddress::root();
        while addr.length() < LEDGER_DEPTH {
            addr = addr.next_or_next_depth();
            let empty_hash = ledger_empty_hash_at_depth(addr.length());
            let expected_hash = self.get_ledger(id).get_hash(addr.clone());
            let downloaded_hash = self.hashes_matrix(id).get(&addr).cloned();

            if expected_hash == Some(empty_hash.0.to_fp().unwrap()) {
                addr = addr.next_depth();
                continue;
            }

            assert_eq!(expected_hash, downloaded_hash, "hash mismatch at: {addr:?}");

            if expected_hash.is_none() {
                addr = addr.next_depth();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mina_p2p_messages::v2::MinaBaseLedgerHash0StableV1;

    use super::*;

    #[test]
    fn test_ledger_hash() {
        IntoIterator::into_iter([
            (
                LedgerAddress::root(),
                "jwDnyiwZ4a3izRefHAKSrt5U5K6gS6p6G58kWdnJ5wYJqHUyaVd",
                "jxt96SwXGrpiyq9AVZ1B7vbt14NwcxXHgnBjArPpfM3dm216oy6",
                "jwq3nCDr8XejL8HKDxR5qVhFJbKoUTGZgtLBZCp3MrqLTnqmjdP",
            ),
            (
                LedgerAddress::root(),
                "jwLYDFqnEzBXjmwKBorWMiRnxYoisU69ZGEvS7g9Bev4WudZsxC",
                "jxoNu92jnreeAXnYxhMChEM22Bf5yAEw8pa7NRNitUnyKvzjcDs",
                "jwq3nCDr8XejL8HKDxR5qVhFJbKoUTGZgtLBZCp3MrqLTnqmjdP",
            ),
        ])
        .map(|(addr, expected_hash, left, right)| {
            let left: LedgerHash = left.parse().unwrap();
            let right: LedgerHash = right.parse().unwrap();
            (addr, expected_hash, left, right)
        })
        .for_each(|(address, expected_hash, left, right)| {
            let hash = ledger_hash(address.length(), left.0.to_field(), right.0.to_field());
            let hash: LedgerHash = MinaBaseLedgerHash0StableV1(hash.into()).into();
            assert_eq!(hash.to_string(), expected_hash);
        });
    }
}

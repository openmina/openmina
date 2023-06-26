use ledger::{AccountIndex, BaseLedger, HashesMatrix, Mask};
use mina_hasher::Fp;
use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};

use super::{LedgerAddress, LedgerId, LEDGER_DEPTH};

fn ledger_hash(depth: usize, left: Fp, right: Fp) -> Fp {
    let param = format!("MinaMklTree{:03}", depth);
    ledger::hash_with_kimchi(param.as_str(), &[left, right])
}

fn validate_children<F>(parent: &LedgerAddress, mut get: F) -> Result<(), ()>
where
    F: FnMut(&LedgerAddress) -> Option<Fp>,
{
    let Some((left, right, expected_hash)) = Some(()).and_then(|_| {
        let left = get(&parent.child_left())?;
        let right = get(&parent.child_right())?;
        let expected_hash = get(&parent)?;
        Some((left, right, expected_hash))
    }) else { return Ok(()) };

    let hash = ledger_hash(parent.length() + 1, left, right);
    match hash == expected_hash {
        true => Ok(()),
        false => Err(()),
    }
}

pub trait LedgerService: redux::Service {
    fn hashes_matrix(&mut self, id: &LedgerId) -> &mut HashesMatrix;
    fn get_ledger(&mut self, id: &LedgerId) -> &mut Mask;

    fn hashes_matrix_set(
        &mut self,
        id: &LedgerId,
        parent: &LedgerAddress,
        hashes: (LedgerHash, LedgerHash),
    ) -> Result<(), ()> {
        let matrix = self.hashes_matrix(id);
        let (left_hash, right_hash) = hashes;
        let (left_fp, right_fp) = (left_hash.0.to_field(), right_hash.0.to_field());
        let left_addr = parent.child_left();
        let right_addr = parent.child_right();

        validate_children(parent, |addr| {
            if addr == &left_addr {
                Some(left_fp)
            } else if addr == &right_addr {
                Some(right_fp)
            } else {
                matrix.get(addr).cloned()
            }
        })?;

        matrix.set(&left_addr, left_fp);
        matrix.set(&right_addr, right_fp);

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
}

//! Coinbase transaction types for the Mina Protocol
//!
//! This module provides types for coinbase transactions, which are
//! system-generated transactions that reward block producers for successfully
//! producing blocks.
//!
//! # Overview
//!
//! When a block producer creates a new block, they receive a coinbase reward.
//! This reward can optionally include a fee transfer to a SNARK worker who
//! provided proofs for transactions in the block.
//!
//! # Types
//!
//! - [`Coinbase`]: The main coinbase transaction containing the reward
//! - [`CoinbaseFeeTransfer`]: Optional fee transfer to a SNARK worker

use mina_signer::CompressedPubKey;

use crate::currency::{Amount, Fee};

/// A fee transfer from a coinbase reward to a SNARK worker.
///
/// When a block contains transactions that were proved by a SNARK worker,
/// the block producer can allocate a portion of the coinbase reward to that
/// worker as compensation for their proof work.
///
/// # Fields
///
/// * `receiver_pk` - The public key of the SNARK worker receiving the fee
/// * `fee` - The fee amount being transferred to the SNARK worker
///
/// # Constraints
///
/// The fee must not exceed the total coinbase amount. This constraint is
/// enforced when creating a [`Coinbase`] transaction.
///
/// # OCaml Reference
///
/// This type corresponds to `Coinbase.Fee_transfer.t` in the OCaml
/// implementation at `src/lib/mina_base/coinbase.ml`.
#[derive(Debug, Clone, PartialEq)]
pub struct CoinbaseFeeTransfer {
    /// The public key of the SNARK worker receiving the fee.
    ///
    /// This is typically the public key that the SNARK worker registered when
    /// submitting their proof work to the network.
    pub receiver_pk: CompressedPubKey,

    /// The fee amount being transferred to the SNARK worker.
    ///
    /// This value is in nanomina (1 MINA = 1,000,000,000 nanomina).
    /// The fee represents compensation for the computational work of
    /// generating zero-knowledge proofs.
    pub fee: Fee,
}

impl CoinbaseFeeTransfer {
    /// Creates a new coinbase fee transfer.
    ///
    /// # Arguments
    ///
    /// * `receiver_pk` - The public key of the SNARK worker receiving the fee
    /// * `fee` - The fee amount to transfer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mina_tx_type::{CoinbaseFeeTransfer, Fee, CompressedPubKey};
    ///
    /// let receiver = CompressedPubKey::from_address("B62...").unwrap();
    /// let fee = Fee::from_u64(10_000_000); // 0.01 MINA
    /// let transfer = CoinbaseFeeTransfer::create(receiver, fee);
    /// ```
    pub fn create(receiver_pk: CompressedPubKey, fee: Fee) -> Self {
        Self { receiver_pk, fee }
    }
}

/// A coinbase transaction rewarding a block producer.
///
/// Coinbase transactions are system-generated transactions that reward the
/// block producer for successfully producing a block. The reward consists of:
///
/// 1. A fixed block reward (currently 720 MINA on mainnet)
/// 2. Optionally, a fee transfer to compensate a SNARK worker
///
/// # Fields
///
/// * `receiver` - The public key of the block producer receiving the reward
/// * `amount` - The total coinbase amount (before any fee transfer deduction)
/// * `fee_transfer` - Optional fee transfer to a SNARK worker
///
/// # Validation
///
/// When creating a coinbase transaction:
/// - If a fee transfer is specified, the fee must not exceed the coinbase
///   amount
/// - If the fee transfer receiver is the same as the coinbase receiver, the
///   fee transfer is removed (as the producer receives the full amount anyway)
///
/// # OCaml Reference
///
/// This type corresponds to `Coinbase.t` in the OCaml implementation at
/// `src/lib/mina_base/coinbase.ml` lines 17-21.
///
/// OCaml reference: src/lib/mina_base/coinbase.ml L:17-21
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
#[derive(Debug, Clone, PartialEq)]
pub struct Coinbase {
    /// The public key of the block producer receiving the coinbase reward.
    ///
    /// This is the public key that the block producer used to stake and
    /// was selected to produce the block.
    pub receiver: CompressedPubKey,

    /// The total coinbase amount before any fee transfer deduction.
    ///
    /// This value is in nanomina (1 MINA = 1,000,000,000 nanomina).
    /// On mainnet, this is typically 720 MINA for a full coinbase.
    pub amount: Amount,

    /// Optional fee transfer to a SNARK worker.
    ///
    /// If present, a portion of the coinbase is transferred to the SNARK
    /// worker who provided proofs for transactions in the block. The fee
    /// is deducted from the coinbase amount when computing the producer's
    /// actual reward.
    ///
    /// This field is `None` when:
    /// - No SNARK work was included in the block
    /// - The SNARK worker is the same as the block producer
    pub fee_transfer: Option<CoinbaseFeeTransfer>,
}

impl Coinbase {
    /// Validates that the coinbase transaction is well-formed.
    ///
    /// A coinbase is valid if:
    /// - There is no fee transfer, OR
    /// - The fee transfer amount does not exceed the coinbase amount
    fn is_valid(&self) -> bool {
        match &self.fee_transfer {
            None => true,
            Some(CoinbaseFeeTransfer { fee, .. }) => Amount::of_fee(fee) <= self.amount,
        }
    }

    /// Creates a new coinbase transaction.
    ///
    /// # Arguments
    ///
    /// * `amount` - The total coinbase amount
    /// * `receiver` - The public key of the block producer
    /// * `fee_transfer` - Optional fee transfer to a SNARK worker
    ///
    /// # Returns
    ///
    /// Returns `Ok(Coinbase)` if the coinbase is valid, or an error message
    /// if the fee transfer exceeds the coinbase amount.
    ///
    /// # Special Cases
    ///
    /// If the fee transfer receiver is the same as the coinbase receiver,
    /// the fee transfer is automatically removed since the producer would
    /// receive the full amount anyway.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mina_tx_type::{Coinbase, CoinbaseFeeTransfer, Amount, Fee, CompressedPubKey};
    ///
    /// let producer = CompressedPubKey::from_address("B62...").unwrap();
    /// let amount = Amount::from_u64(720_000_000_000); // 720 MINA
    ///
    /// // Create a coinbase with no fee transfer
    /// let cb = Coinbase::create(amount, producer.clone(), None).unwrap();
    ///
    /// // Create a coinbase with a fee transfer to a SNARK worker
    /// let worker = CompressedPubKey::from_address("B62...").unwrap();
    /// let fee_transfer = CoinbaseFeeTransfer::create(worker, Fee::from_u64(10_000_000));
    /// let cb = Coinbase::create(amount, producer, Some(fee_transfer)).unwrap();
    /// ```
    pub fn create(
        amount: Amount,
        receiver: CompressedPubKey,
        fee_transfer: Option<CoinbaseFeeTransfer>,
    ) -> Result<Coinbase, String> {
        let mut this = Self {
            receiver: receiver.clone(),
            amount,
            fee_transfer,
        };

        if this.is_valid() {
            // If the fee transfer receiver is the same as the coinbase receiver,
            // remove the fee transfer since the producer gets everything anyway
            let adjusted_fee_transfer = this.fee_transfer.as_ref().and_then(|ft| {
                if receiver != ft.receiver_pk {
                    Some(ft.clone())
                } else {
                    None
                }
            });
            this.fee_transfer = adjusted_fee_transfer;
            Ok(this)
        } else {
            Err("Coinbase.create: invalid coinbase".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a test public key
    fn test_pubkey() -> CompressedPubKey {
        CompressedPubKey::from_address("B62qiy32p8kAKnny8ZFwoMhYpBppM1DWVCqAPBYNcXnsAHhnfAAuXgg")
            .unwrap()
    }

    fn test_pubkey_2() -> CompressedPubKey {
        CompressedPubKey::from_address("B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV")
            .unwrap()
    }

    #[test]
    fn test_coinbase_fee_transfer_create() {
        let receiver = test_pubkey();
        let fee = Fee::from_u64(10_000_000);
        let transfer = CoinbaseFeeTransfer::create(receiver.clone(), fee);

        assert_eq!(transfer.receiver_pk, receiver);
        assert_eq!(transfer.fee, fee);
    }

    #[test]
    fn test_coinbase_create_no_fee_transfer() {
        let producer = test_pubkey();
        let amount = Amount::from_u64(720_000_000_000);

        let cb = Coinbase::create(amount, producer.clone(), None).unwrap();

        assert_eq!(cb.receiver, producer);
        assert_eq!(cb.amount, amount);
        assert!(cb.fee_transfer.is_none());
    }

    #[test]
    fn test_coinbase_create_with_fee_transfer() {
        let producer = test_pubkey();
        let worker = test_pubkey_2();
        let amount = Amount::from_u64(720_000_000_000);
        let fee = Fee::from_u64(10_000_000);

        let fee_transfer = CoinbaseFeeTransfer::create(worker.clone(), fee);
        let cb = Coinbase::create(amount, producer.clone(), Some(fee_transfer)).unwrap();

        assert_eq!(cb.receiver, producer);
        assert_eq!(cb.amount, amount);
        assert!(cb.fee_transfer.is_some());
        let ft = cb.fee_transfer.unwrap();
        assert_eq!(ft.receiver_pk, worker);
        assert_eq!(ft.fee, fee);
    }

    #[test]
    fn test_coinbase_removes_self_fee_transfer() {
        let producer = test_pubkey();
        let amount = Amount::from_u64(720_000_000_000);
        let fee = Fee::from_u64(10_000_000);

        // Fee transfer to self should be removed
        let fee_transfer = CoinbaseFeeTransfer::create(producer.clone(), fee);
        let cb = Coinbase::create(amount, producer.clone(), Some(fee_transfer)).unwrap();

        assert!(cb.fee_transfer.is_none());
    }

    #[test]
    fn test_coinbase_invalid_fee_exceeds_amount() {
        let producer = test_pubkey();
        let worker = test_pubkey_2();
        let amount = Amount::from_u64(100);
        let fee = Fee::from_u64(200); // Fee exceeds amount

        let fee_transfer = CoinbaseFeeTransfer::create(worker, fee);
        let result = Coinbase::create(amount, producer, Some(fee_transfer));

        assert!(result.is_err());
    }
}

use crate::{bigint, versioned::Versioned};

pub type StateHashV1 = bigint::BigInt;
pub type LedgerHashV1 = bigint::BigInt;
pub type StateBodyHashV1 = bigint::BigInt;

pub type StateHashV1Versioned = Versioned<StateHashV1, 1>;
pub type StateBodyHashV1Versioned = Versioned<StateBodyHashV1, 1>;
pub type LedgerHashV1Versioned = Versioned<LedgerHashV1, 1>;

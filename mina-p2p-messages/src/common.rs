use crate::{bigint, versioned::Versioned};

pub type StateHashV1 = bigint::BigInt;
pub type LedgerHashV1 = bigint::BigInt;
pub type StateBodyHashV1 = bigint::BigInt;

pub type StateHashV1Binable = Versioned<StateHashV1, 1>;
pub type StateBodyHashV1Binable = Versioned<StateBodyHashV1, 1>;
pub type LedgerHashV1Binable = Versioned<LedgerHashV1, 1>;

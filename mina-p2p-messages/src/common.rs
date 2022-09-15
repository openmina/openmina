use crate::{bigint, versioned::Versioned};

pub type StateHashV1Binable = Versioned<bigint::BigInt, 1>;
pub type StateBodyHashV1Binable = Versioned<bigint::BigInt, 1>;
pub type LedgerHashV1Binable = Versioned<bigint::BigInt, 1>;

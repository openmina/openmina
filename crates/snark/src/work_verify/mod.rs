//! # SNARK Work Verification State Machine
//!
//! This module manages the verification of SNARK work proofs submitted by
//! external SNARK workers. It validates computational work that helps maintain
//! the blockchain's security and scalability.
//!
//! ## Overview
//!
//! SNARK work verification handles:
//! - **Transaction SNARK proofs**: Proofs for transaction validity
//! - **Merge proofs**: Proofs combining multiple transaction proofs

mod snark_work_verify_state;
pub use snark_work_verify_state::*;

mod snark_work_verify_actions;
pub use snark_work_verify_actions::*;

mod snark_work_verify_reducer;
pub use snark_work_verify_reducer::reducer;

pub use crate::work_verify_effectful::{
    SnarkWorkVerifyError, SnarkWorkVerifyId, SnarkWorkVerifyIdType,
};

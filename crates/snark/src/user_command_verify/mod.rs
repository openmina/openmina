//! # User Command Verification State Machine
//!
//! This module handles the verification of user commands and zkApp transactions
//! within the Mina protocol. It manages SNARK validation for transactions
//! before they are included in blocks.
//!
//! ## Overview
//!
//! User command verification validates:
//! - **Payment transactions**: Simple value transfers between accounts
//! - **Delegation commands**: Stake delegation operations
//! - **zkApp transactions**
//!
//! ## Verification Process
//!
//! The verification process includes:
//! - Signature validation for transaction authorization
//! - Proof verification for zkApp transactions
//! - Account state consistency checks
//! - Fee and nonce validation

mod snark_user_command_verify_state;
pub use snark_user_command_verify_state::*;

mod snark_user_command_verify_actions;
pub use snark_user_command_verify_actions::*;

mod snark_user_command_verify_reducer;
pub use snark_user_command_verify_reducer::reducer;

pub use crate::user_command_verify_effectful::{
    SnarkUserCommandVerifyError, SnarkUserCommandVerifyId, SnarkUserCommandVerifyIdType,
};

use mina_signer::CompressedPubKey;

use crate::scan_state::{
    scan_state::ConstraintConstants,
    transaction_logic::{valid, UserCommand},
};

use super::diff;

impl diff::Diff {
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/staged_ledger/pre_diff_info.ml#L457
    pub fn get<F>(
        &self,
        check: F,
        constraint_constants: &ConstraintConstants,
        coinbase_receiver: CompressedPubKey,
        supercharge_coinbase: bool,
    ) where
        F: Fn(Vec<UserCommand>) -> Result<Vec<valid::UserCommand>, ()>,
    {
    }
}

// (* TODO: This is important *)
// let get ~check ~constraint_constants ~coinbase_receiver ~supercharge_coinbase t
//     =
//   let open Async in
//   match%map validate_commands t ~check with
//   | Error e ->
//       Error (Error.Unexpected e)
//   | Ok (Error e) ->
//       Error (Error.Verification_failed e)
//   | Ok (Ok diff) ->
//       get' ~constraint_constants ~forget:User_command.forget_check
//         ~diff:diff.diff ~coinbase_receiver
//         ~coinbase_amount:
//           (Staged_ledger_diff.With_valid_signatures.coinbase
//              ~constraint_constants ~supercharge_coinbase diff )

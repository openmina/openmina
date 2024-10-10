use std::sync::Arc;

use mina_p2p_messages::v2::{MinaBaseProofStableV2, TransactionSnarkProofStableV2};

// NOTE: moved to mina_p2p_messages crate
/// Value of `Proof.transaction_dummy` when we run `dune runtest src/lib/staged_ledger -f`
/// The file was generated this way:
///
/// let dummy = Proof.transaction_dummy in
///
/// let buf = Bigstring.create (Proof.Stable.V2.bin_size_t dummy) in
/// ignore (Proof.Stable.V2.bin_write_t buf ~pos:0 dummy : int) ;
/// let bytes = Bigstring.to_bytes buf in
///
/// let explode s = List.init (String.length s) ~f:(fun i -> String.get s i) in
///
/// let s = (String.concat ~sep:"," (List.map (explode (Bytes.to_string bytes)) ~f:(fun b -> string_of_int (Char.to_int b)))) in
///
/// Core.Printf.eprintf !"dummy proof= %{sexp: Proof.t}\n%!" dummy;
/// Core.Printf.eprintf !"dummy proof= %s\n%!" s;
pub fn dummy_transaction_proof() -> Arc<TransactionSnarkProofStableV2> {
    mina_p2p_messages::v2::dummy_transaction_proof()
}

/// Value of `Proof.blockchain_dummy`
pub fn dummy_blockchain_proof() -> Arc<MinaBaseProofStableV2> {
    mina_p2p_messages::v2::dummy_blockchain_proof()
}

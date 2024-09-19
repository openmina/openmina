use std::sync::Arc;

use binprot::BinProtRead;

use super::{MinaBaseProofStableV2, TransactionSnarkProofStableV2};

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
    lazy_static::lazy_static! {
        static ref DUMMY_PROOF: Arc<TransactionSnarkProofStableV2> = {
            let bytes = include_bytes!("dummy_transaction_proof.bin");
            TransactionSnarkProofStableV2::binprot_read(&mut bytes.as_slice())
                .unwrap()
                .into()
        };
    }

    DUMMY_PROOF.clone()
}

/// Value of `Proof.blockchain_dummy`
pub fn dummy_blockchain_proof() -> Arc<MinaBaseProofStableV2> {
    lazy_static::lazy_static! {
        static ref DUMMY_PROOF: Arc<MinaBaseProofStableV2> = {
            let bytes = include_bytes!("dummy_blockchain_proof.bin");
            MinaBaseProofStableV2::binprot_read(&mut bytes.as_slice())
                .unwrap()
                .into()
        };
    }

    DUMMY_PROOF.clone()
}

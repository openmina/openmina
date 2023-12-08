use std::sync::Arc;

use mina_p2p_messages::binprot::BinProtRead;
use mina_p2p_messages::v2::MinaBaseProofStableV2;
use mina_p2p_messages::v2::PicklesProofProofsVerifiedMaxStableV2;
use mina_p2p_messages::v2::TransactionSnarkProofStableV2;

#[cfg(test)]
use crate::VerificationKey;

#[cfg(test)]
pub mod for_tests;

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

/// Value of `vk` when we run `dune runtest src/lib/staged_ledger -f`
/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/staged_ledger/staged_ledger.ml#L2083
///
/// The file was generated this way:
///
/// let buf = Bigstring.create (Side_loaded_verification_key.Stable.V2.bin_size_t vk.data) in
/// ignore (Side_loaded_verification_key.Stable.V2.bin_write_t buf ~pos:0 vk.data : int) ;
/// let bytes = Bigstring.to_bytes buf in
/// let explode s = List.init (String.length s) ~f:(fun i -> String.get s i) in
/// let s = (String.concat ~sep:"," (List.map (explode (Bytes.to_string bytes)) ~f:(fun b -> string_of_int (Char.to_int b)))) in
///
/// Core.Printf.eprintf !"vk=%{sexp: (Side_loaded_verification_key.t, Frozen_ledger_hash.t) With_hash.t}\n%!" vk;
/// Core.Printf.eprintf !"vk_binprot=[%s]\n%!" s;
#[cfg(test)] // Used for tests only
pub fn trivial_verification_key() -> VerificationKey {
    use mina_p2p_messages::v2::MinaBaseVerificationKeyWireStableV1;

    let mut cursor = std::io::Cursor::new(include_bytes!("trivial_vk.bin"));
    let vk = MinaBaseVerificationKeyWireStableV1::binprot_read(&mut cursor).unwrap();

    let vk: VerificationKey = (&vk).into();
    vk
}

/// Value of a dummy proof when we run `dune runtest src/lib/staged_ledger -f`
/// https://github.com/MinaProtocol/mina/blob/d7dad23d8ea2052f515f5d55d187788fe0701c7f/src/lib/mina_base/control.ml#L94
///
/// The file was generated this way:
///
/// let buf = Bigstring.create (Pickles.Proof.Proofs_verified_2.Stable.V2.bin_size_t proof) in
/// ignore (Pickles.Proof.Proofs_verified_2.Stable.V2.bin_write_t buf ~pos:0 proof : int) ;
/// let bytes = Bigstring.to_bytes buf in
/// let explode s = List.init (String.length s) ~f:(fun i -> String.get s i) in
/// let s = (String.concat ~sep:"," (List.map (explode (Bytes.to_string bytes)) ~f:(fun b -> string_of_int (Char.to_int b)))) in
///
/// Printf.eprintf !"proof_sexp=%{sexp: Pickles.Proof.Proofs_verified_2.Stable.V2.t}\n%!" proof;
/// Printf.eprintf !"proof_binprot=[%s]\n%!" s;
pub fn sideloaded_proof() -> Arc<PicklesProofProofsVerifiedMaxStableV2> {
    let mut cursor = std::io::Cursor::new(include_bytes!("sideloaded_proof.bin"));
    let proof = PicklesProofProofsVerifiedMaxStableV2::binprot_read(&mut cursor).unwrap();

    proof.into()
}

use binprot::{BinProtRead, BinProtShape};
use mina_p2p_messages::v2::MinaBaseUserCommandStableV2;

use crate::scan_state::transaction_logic::valid;

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
pub fn list_of_cmds() -> Vec<valid::UserCommand> {
    let mut cursor = std::io::Cursor::new(include_bytes!("cmds.bin"));
    let cmds: Vec<MinaBaseUserCommandStableV2> =
        Vec::<MinaBaseUserCommandStableV2>::binprot_read(&mut cursor).unwrap();

    cmds.iter().map(|cmd| cmd.into()).collect()

    // todo!()

    // cmds
}

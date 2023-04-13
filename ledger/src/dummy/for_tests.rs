use binprot::BinProtRead;
use mina_p2p_messages::v2::MinaBaseUserCommandStableV2;

use crate::scan_state::transaction_logic::valid;

/// 882 signed commands (non-zkapps) generated this way in `src/staged_ledger/staged_ledger.ml`:
///
/// let cmds_per_iter = [
///     126;
///     126;
///     126;
///     126;
///     126;
///     126;
///     126;
///   ] in
/// let total_cmds = List.fold cmds_per_iter ~init:0 ~f:( + ) in
/// let%bind cmds =
///   User_command.Valid.Gen.sequence ~length:total_cmds ~sign_type:`Real
///     ledger_init_state
/// in
/// assert (List.length cmds = total_cmds) ;
///
/// (* let buf = Bigstring.create (List.bin_size_t User_command.Valid.Stable.V2.bin_size_t cmds) in *)
/// (* ignore (List.bin_write_t User_command.Valid.Stable.V2.bin_write_t buf ~pos:0 cmds : int) ; *)
/// (* let bytes = Bigstring.to_bytes buf in *)
/// (* let explode s = List.init (String.length s) ~f:(fun i -> String.get s i) in *)
/// (* let s = (String.concat ~sep:"," (List.map (explode (Bytes.to_string bytes)) ~f:(fun b -> string_of_int (Char.to_int b)))) in *)
/// (* Core.Printf.eprintf !"cmds_binprot=[%s]\n%!" s; *)
///
/// (* return (ledger_init_state, cmds, a) *)
///
/// let cmds_per_iter = [126; 126; 126; 1] in
/// (* let cmds_per_iter = [126; 126; 126; 126] in *)
/// let total_cmds = List.fold cmds_per_iter ~init:0 ~f:( + ) in
/// let cmds = List.take cmds total_cmds in
/// Core.Printf.eprintf !"cmds=%{sexp: User_command.Valid.t list}\n%!" cmds;
pub fn list_of_cmds() -> Vec<valid::UserCommand> {
    let mut cursor = std::io::Cursor::new(include_bytes!("cmds.bin"));
    let cmds: Vec<MinaBaseUserCommandStableV2> =
        Vec::<MinaBaseUserCommandStableV2>::binprot_read(&mut cursor).unwrap();

    cmds.iter().map(|cmd| cmd.into()).collect()
}

/// Core.Printf.eprintf
///   !"PROTOCOL_STATE=%{sexp: Mina_state.Protocol_state.value}\n%!" state ;
///
/// let buf = Bigstring.create (Mina_state.Protocol_state.Value.Stable.V2.bin_size_t state) in
/// ignore (Mina_state.Protocol_state.Value.Stable.V2.bin_write_t buf ~pos:0 state : int) ;
/// let bytes = Bigstring.to_bytes buf in
/// let explode s = List.init (String.length s) ~f:(fun i -> String.get s i) in
/// let s = (String.concat ~sep:"," (List.map (explode (Bytes.to_string bytes)) ~f:(fun b -> string_of_int (Char.to_int b)))) in
///
/// Core.Printf.eprintf !"state_binprot=[%s]\n%!" s;
///
pub fn dummy_protocol_state() -> mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2 {
    let mut cursor = std::io::Cursor::new(include_bytes!("protocol_state.bin"));
    mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2::binprot_read(&mut cursor).unwrap()
}

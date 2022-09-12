
type database
type addr = string
type account = bytes
type token_id = bytes
type account_id = bytes
type pubkey = bytes

type rust_dberror =
  | Account_location_not_found
  | Out_of_leaves
  | Malformed_database

module Rust = struct
  external database_create : int -> database = "rust_database_create"
  external database_get_uuid : database -> string = "rust_database_get_uuid"
  external database_depth : database -> int = "rust_database_depth"
  external database_create_checkpoint : database -> database = "rust_database_create_checkpoint"
  external database_make_checkpoint : database -> unit = "rust_database_make_checkpoint"
  external database_close : database -> unit = "rust_database_close"
  external database_get : database -> addr -> account option = "rust_database_get"
  external database_get_batch : database -> addr list -> (addr * (account option)) list = "rust_database_get_batch"
  external database_get_list : database -> bytes list = "rust_database_get_list"
  external database_accounts : database -> bytes list = "rust_database_accounts"
  external database_get_inner_hash_at_addr : database -> addr -> bytes = "rust_database_get_inner_hash_at_addr"
  external database_set_inner_hash_at_addr : database -> addr -> bytes -> unit = "rust_database_set_inner_hash_at_addr"
  external database_get_at_index : database -> int -> account = "rust_database_get_at_index"
  external database_iter : database -> (int -> bytes -> unit) -> unit = "rust_database_iter"
  external database_location_of_account : database -> account_id -> addr option = "rust_database_location_of_account"
  external database_location_of_account_batch : database -> account_id list -> (account_id * (addr option)) list = "rust_database_location_of_account_batch"
  external database_last_filled : database -> addr option = "rust_database_last_filled"
  external database_token_owners : database -> bytes list = "rust_database_token_owners"
  external database_token_owner : database -> token_id -> account_id option = "rust_database_token_owner"
  external database_tokens : database -> pubkey -> token_id list = "rust_database_tokens"
  external database_set : database -> addr -> account -> unit = "rust_database_set"
  external database_index_of_account : database -> account_id -> int = "rust_database_index_of_account"
  external database_set_at_index : database -> int -> account -> unit = "rust_database_set_at_index"
  external database_get_or_create_account : database -> account_id -> account -> (([ `Added | `Existed ] * addr), rust_dberror) result = "rust_database_get_or_create_account"
  external database_num_accounts : database -> int = "rust_database_num_accounts"
  (* external database_fold_with_ignored_accounts : database -> bytes list -> bytes -> (bytes -> unit) -> bytes = "rust_database_fold_with_ignored_accounts" *)
  (* external database_fold : database -> bytes -> (bytes -> unit) -> bytes = "rust_database_fold" *)
  (* external database_fold_until : database -> bytes -> (bytes -> bool) -> bytes = "rust_database_fold_until" *)
  external database_merkle_root : database -> bytes = "rust_database_merkle_root"
  external database_remove_accounts : database -> account_id list -> unit = "rust_database_remove_accounts"
  external database_merkle_path : database -> addr -> bytes list = "rust_database_merkle_path"
  external database_merkle_path_at_addr : database -> bytes -> bytes list = "rust_database_merkle_path_at_addr"
  external database_merkle_path_at_index : database -> int -> bytes list = "rust_database_merkle_path_at_index"
  external database_set_all_accounts_rooted_at : database -> addr -> bytes list -> unit = "rust_database_set_all_accounts_rooted_at"
  external database_set_batch_accounts : database -> (addr * account) list -> unit = "rust_database_set_batch_accounts"
  external database_get_all_accounts_rooted_at : database -> addr -> (addr * account) list = "rust_database_get_all_accounts_rooted_at"

  (* TODO: Make those method *)
  external database_foldi : database -> (addr -> bytes -> unit) -> unit = "rust_database_foldi"
  external database_foldi_with_ignored_accounts : database -> account_id list -> (addr -> bytes -> unit) -> unit = "rust_database_foldi_with_ignored_accounts"
end

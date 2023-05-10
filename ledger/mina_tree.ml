
type ondisk_database
type ondisk_batch
type database
type mask

type addr = string
type account = bytes
type token_id = bytes
type account_id = bytes
type pubkey = bytes

type ondisk_key = Core.Bigstring.t
type ondisk_value = Core.Bigstring.t

type rust_dberror =
  | Account_location_not_found
  | Out_of_leaves
  | Malformed_database

type rust_path = [ `Left of bytes | `Right of bytes ]

type rust_grandchildren = [ `Check | `Recursive | `I_promise_I_am_reparenting_this_mask ]

module Rust = struct

  (* type unattached_mask *)
  (* type attached_mask *)
  (* type any_mask *)

  external transaction_fuzzer : (bytes -> bytes) -> (bytes -> bytes) -> unit = "rust_transaction_fuzzer"

  external mask_create : int -> mask = "rust_mask_create"
  external mask_get_directory : mask -> string option = "rust_mask_get_directory"
  external mask_get_uuid : mask -> string = "rust_mask_get_uuid"
  external mask_depth : mask -> int = "rust_mask_depth"
  external mask_close : mask -> unit = "rust_mask_close"
  external mask_get : mask -> addr -> account option = "rust_mask_get"
  external mask_get_batch : mask -> addr list -> (addr * (account option)) list = "rust_mask_get_batch"
  external mask_get_list : mask -> bytes list = "rust_mask_get_list"
  external mask_accounts : mask -> bytes list = "rust_mask_accounts"
  external mask_get_inner_hash_at_addr : mask -> addr -> bytes = "rust_mask_get_inner_hash_at_addr"
  external mask_set_inner_hash_at_addr : mask -> addr -> bytes -> unit = "rust_mask_set_inner_hash_at_addr"
  external mask_get_at_index : mask -> int -> account = "rust_mask_get_at_index"
  external mask_iter : mask -> (int -> bytes -> unit) -> unit = "rust_mask_iter"
  external mask_location_of_account : mask -> account_id -> addr option = "rust_mask_location_of_account"
  external mask_location_of_account_batch : mask -> account_id list -> (account_id * (addr option)) list = "rust_mask_location_of_account_batch"
  external mask_last_filled : mask -> addr option = "rust_mask_last_filled"
  external mask_token_owners : mask -> bytes list = "rust_mask_token_owners"
  external mask_token_owner : mask -> token_id -> account_id option = "rust_mask_token_owner"
  external mask_tokens : mask -> pubkey -> token_id list = "rust_mask_tokens"
  external mask_set : mask -> addr -> account -> unit = "rust_mask_set"
  external mask_index_of_account : mask -> account_id -> int = "rust_mask_index_of_account"
  external mask_set_at_index : mask -> int -> account -> unit = "rust_mask_set_at_index"
  external mask_get_or_create_account : mask -> account_id -> account -> (([ `Added | `Existed ] * addr), rust_dberror) result = "rust_mask_get_or_create_account"
  external mask_num_accounts : mask -> int = "rust_mask_num_accounts"
  (* external mask_fold_with_ignored_accounts : mask -> bytes list -> bytes -> (bytes -> unit) -> bytes = "rust_mask_fold_with_ignored_accounts" *)
  (* external mask_fold : mask -> bytes -> (bytes -> unit) -> bytes = "rust_mask_fold" *)
  (* external mask_fold_until : mask -> bytes -> (bytes -> bool) -> bytes = "rust_mask_fold_until" *)
  external mask_merkle_root : mask -> bytes = "rust_mask_merkle_root"
  external mask_remove_accounts : mask -> account_id list -> unit = "rust_mask_remove_accounts"
  external mask_merkle_path : mask -> addr -> rust_path list = "rust_mask_merkle_path"
  external mask_merkle_path_at_addr : mask -> addr -> rust_path list = "rust_mask_merkle_path_at_addr"
  external mask_merkle_path_at_index : mask -> int -> rust_path list = "rust_mask_merkle_path_at_index"
  external mask_set_all_accounts_rooted_at : mask -> addr -> bytes list -> unit = "rust_mask_set_all_accounts_rooted_at"
  external mask_set_batch_accounts : mask -> (addr * account) list -> unit = "rust_mask_set_batch_accounts"
  external mask_get_all_accounts_rooted_at : mask -> addr -> (addr * account) list = "rust_mask_get_all_accounts_rooted_at"
  (* TODO: Make those method *)
  external mask_foldi : mask -> (addr -> bytes -> unit) -> unit = "rust_mask_foldi"
  external mask_foldi_with_ignored_accounts : mask -> account_id list -> (addr -> bytes -> unit) -> unit = "rust_mask_foldi_with_ignored_accounts"

  (* external mask_get_parent : mask -> mask = "rust_mask_get_parent" *)
  (* external mask_get_hash : mask -> addr -> bytes = "rust_mask_get_hash" *)
  external mask_commit : mask -> unit = "rust_mask_commit"
  external mask_copy : mask -> mask = "rust_mask_copy"

  (* external mask_set_parent : mask -> mask -> mask = "rust_mask_set_parent" *)
  (* external mask_register_mask : mask -> mask -> mask = "rust_mask_register_mask" *)
  (* external mask_register_mask : any_mask -> unattached_mask -> attached_mask = "rust_mask_register_mask" *)
  (* external mask_unregister_mask : mask -> rust_grandchildren -> mask = "rust_mask_unregister_mask"
   * external mask_remove_and_reparent : mask -> mask -> unit = "rust_mask_remove_and_reparent" *)

  external database_create : int -> string option -> database = "rust_database_create"
  external database_get_uuid : database -> string = "rust_database_get_uuid"
  external database_get_directory : database -> string option = "rust_database_get_directory"
  external database_depth : database -> int = "rust_database_depth"
  external database_create_checkpoint : database -> string -> database = "rust_database_create_checkpoint"
  external database_make_checkpoint : database -> string -> unit = "rust_database_make_checkpoint"
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
  external database_merkle_path : database -> addr -> rust_path list = "rust_database_merkle_path"
  external database_merkle_path_at_addr : database -> addr -> rust_path list = "rust_database_merkle_path_at_addr"
  external database_merkle_path_at_index : database -> int -> rust_path list = "rust_database_merkle_path_at_index"
  external database_set_all_accounts_rooted_at : database -> addr -> bytes list -> unit = "rust_database_set_all_accounts_rooted_at"
  external database_set_batch_accounts : database -> (addr * account) list -> unit = "rust_database_set_batch_accounts"
  external database_get_all_accounts_rooted_at : database -> addr -> (addr * account) list = "rust_database_get_all_accounts_rooted_at"

  (* TODO: Make those method *)
  external database_foldi : database -> (addr -> bytes -> unit) -> unit = "rust_database_foldi"
  external database_foldi_with_ignored_accounts : database -> account_id list -> (addr -> bytes -> unit) -> unit = "rust_database_foldi_with_ignored_accounts"

  external ondisk_database_create : string -> ondisk_database = "rust_ondisk_database_create"
  external ondisk_database_create_checkpoint : ondisk_database -> string -> ondisk_database = "rust_ondisk_database_create_checkpoint"
  external ondisk_database_make_checkpoint : ondisk_database -> string -> unit = "rust_ondisk_database_make_checkpoint"
  external ondisk_database_get_uuid : ondisk_database -> string = "rust_ondisk_database_get_uuid"
  external ondisk_database_close : ondisk_database -> unit = "rust_ondisk_database_close"
  external ondisk_database_get : ondisk_database -> ondisk_key -> ondisk_value option = "rust_ondisk_database_get"
  external ondisk_database_get_batch : ondisk_database -> ondisk_key list -> ondisk_value option list = "rust_ondisk_database_get_batch"
  external ondisk_database_set : ondisk_database -> ondisk_key -> ondisk_value -> unit = "rust_ondisk_database_set"
  external ondisk_database_set_batch : ondisk_database -> 'key list -> (ondisk_key * ondisk_value) list -> unit = "rust_ondisk_database_set_batch"
  external ondisk_database_remove : ondisk_database -> ondisk_key -> unit = "rust_ondisk_database_remove"
  external ondisk_database_to_alist : ondisk_database -> (ondisk_key * ondisk_value) list = "rust_ondisk_database_to_alist"

  external ondisk_database_batch_create : unit -> ondisk_batch = "rust_ondisk_database_batch_create"
  external ondisk_database_batch_set : ondisk_batch -> ondisk_key -> ondisk_value -> unit = "rust_ondisk_database_batch_set"
  external ondisk_database_batch_remove : ondisk_batch -> ondisk_key -> unit = "rust_ondisk_database_batch_remove"
  external ondisk_database_batch_run : ondisk_database -> ondisk_batch -> unit = "rust_ondisk_database_batch_run"

  (* For testing *)
  external test_random_accounts : (bytes -> bytes) -> unit = "rust_test_random_accounts"
  external test_random_account_updates : (bytes -> bytes) -> unit = "rust_test_random_account_updates"
  external get_random_account : (bytes -> unit) -> bytes = "rust_get_random_account"
end

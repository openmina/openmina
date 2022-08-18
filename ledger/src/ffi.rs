use std::{str::FromStr, sync::Mutex};

use mina_hasher::Fp;
use ocaml_interop::{ocaml_export, OCaml, OCamlBytes, OCamlRef, ToOCaml};
use once_cell::sync::Lazy;

use crate::{account::Account, tree::Database, tree_version::V2};

static DATABASE: Lazy<Mutex<Database<V2>>> = Lazy::new(|| Mutex::new(Database::create(30)));

ocaml_export! {
    fn rust_add_account(
        rt,
        account: OCamlRef<OCamlBytes>,
    ) {
        println!("RUST BEGIN");
        let account_ref = rt.get(account);
        let account = account_ref.as_bytes();

        let account: Account = serde_binprot::from_slice(account).unwrap();

        println!("account={:?}", account);
        println!("account_hash={:?}", account.hash().to_string());

        println!("RUST END 1");
        OCaml::unit()
    }

    fn rust_add_account_with_hash(
        rt,
        account: OCamlRef<OCamlBytes>,
        hash: OCamlRef<String>,
    ) {
        // println!("RUST BEGIN");
        let account_ref = rt.get(account);
        let account = account_ref.as_bytes();
        let account_bytes = account;
        let _account_len = account.len();
        let hash: String = hash.to_rust(rt);
        let hash = Fp::from_str(&hash).unwrap();

        let account: Account = serde_binprot::from_slice(account).unwrap();
        let account_hash = account.hash();

        if hash != account_hash {
            println!("different hash ! bytes={:?}", account_bytes);
            println!("ocaml_hash={:?}", hash.to_string());
            println!("rust_hash ={:?}", account_hash.to_string());
            assert_eq!(hash, account_hash);
        }

        // println!("hash={:?}", hash.to_string());
        // println!("provided={:?}", hash.to_string());
        // println!("computed={:?}", account_hash.to_string());

        let ser = serde_binprot::to_vec(&account).unwrap();

        // println!("from_ocaml={:?}", account_bytes);
        // println!("rust_ocaml={:?}", ser);

        // assert_eq!(account_len, ser.len());

        let account2: Account = serde_binprot::from_slice(&ser).unwrap();
        let account_hash2 = account2.hash();
        assert_eq!(account_hash, account_hash2);

        // println!("account={:?}", account);
        // println!("account_hash={:?}", account.hash().to_string());

        let mut db = DATABASE.lock().unwrap();
        db.create_account((), account).unwrap();

        // println!("RUST END");
        OCaml::unit()
    }

    fn rust_root_hash(rt, ocaml_hash: OCamlRef<String>) {
        let db = DATABASE.lock().unwrap();
        let hash = db.root_hash();

        let ocaml_hash: String = ocaml_hash.to_rust(rt);
        let ocaml_hash = Fp::from_str(&ocaml_hash).unwrap();

        println!("naccounts ={:?}", db.naccounts());
        println!("rust_root_hash ={:?}", hash.to_string());
        println!("ocaml_root_hash={:?}", ocaml_hash.to_string());

        assert_eq!(hash, ocaml_hash);

        OCaml::unit()
    }

    fn rust_random_account(rt, _unused: OCamlRef<String>) -> OCaml<OCamlBytes> {
        let res = impl_rust_random_account();
        // println!("rust_random_account begin");

        // // let account = Account::rand();
        // // let ser = serde_binprot::to_vec(&account).unwrap();

        // let ser: Vec<u8> = vec![178, 29, 73, 50, 85, 80, 131, 166, 53, 11, 48, 224, 103, 89, 161, 207, 149, 31, 170, 21, 165, 181, 94, 18, 149, 177, 54, 71, 185, 77, 109, 49, 1, 144, 247, 164, 171, 110, 24, 3, 12, 25, 163, 63, 125, 83, 66, 174, 2, 160, 62, 45, 137, 185, 47, 16, 129, 145, 190, 203, 124, 35, 119, 251, 26, 1, 1, 6, 49, 50, 56, 54, 56, 56, 252, 29, 154, 218, 214, 79, 98, 177, 181, 253, 181, 152, 127, 0, 145, 177, 91, 155, 59, 239, 161, 174, 217, 42, 201, 30, 46, 11, 187, 88, 49, 5, 111, 254, 222, 87, 42, 45, 90, 1, 236, 173, 205, 215, 241, 20, 0, 77, 12, 197, 234, 69, 202, 22, 55, 50, 183, 255, 238, 8, 29, 79, 199, 92, 12, 146, 223, 105, 45, 135, 77, 89, 73, 141, 11, 137, 28, 54, 21, 0, 1, 4, 4, 1, 0, 4, 3, 4, 3, 2, 3, 0, 6, 49, 49, 56, 54, 54, 51];

        // let mut account2: Account = serde_binprot::from_slice(&ser).unwrap();

        // let account_hash2 = account2.hash();

        // println!("HASH2={:?}", account_hash2.to_string());

        // let ser = serde_binprot::to_vec(&account2).unwrap();


        // println!("rust_random_account end");

        res.to_ocaml(rt)
    }
}

fn impl_rust_random_account() -> Vec<u8> {
    // println!("rust_random_account begin");

    let account = Account::rand();
    let ser = serde_binprot::to_vec(&account).unwrap();

    // let ser: Vec<u8> = vec![
    //     178, 29, 73, 50, 85, 80, 131, 166, 53, 11, 48, 224, 103, 89, 161, 207, 149, 31, 170, 21,
    //     165, 181, 94, 18, 149, 177, 54, 71, 185, 77, 109, 49, 1, 144, 247, 164, 171, 110, 24, 3,
    //     12, 25, 163, 63, 125, 83, 66, 174, 2, 160, 62, 45, 137, 185, 47, 16, 129, 145, 190, 203,
    //     124, 35, 119, 251, 26, 1, 1, 6, 49, 50, 56, 54, 56, 56, 252, 29, 154, 218, 214, 79, 98,
    //     177, 181, 253, 181, 152, 127, 0, 145, 177, 91, 155, 59, 239, 161, 174, 217, 42, 201, 30,
    //     46, 11, 187, 88, 49, 5, 111, 254, 222, 87, 42, 45, 90, 1, 236, 173, 205, 215, 241, 20, 0,
    //     77, 12, 197, 234, 69, 202, 22, 55, 50, 183, 255, 238, 8, 29, 79, 199, 92, 12, 146, 223,
    //     105, 45, 135, 77, 89, 73, 141, 11, 137, 28, 54, 21, 0, 1, 4, 4, 1, 0, 4, 3, 4, 3, 2, 3, 0,
    //     6, 49, 49, 56, 54, 54, 51,
    // ];

    // let account: Account = serde_binprot::from_slice(&ser).unwrap();

    // // account2.permissions = Permissions::user_default();

    // // let account_hash2 = account2.hash();

    // // println!("HASH2={:?}", account_hash2.to_string());

    // // let mut account2 = Account::empty();

    // // account2.public_key = account.public_key;
    // // account2.token_id = account.token_id;
    // // // account2.token_permissions = account.token_permissions;
    // // account2.token_permissions = TokenPermissions::TokenOwned { disable_new_accounts: false };

    // // println!("ACCOUNT={:#?}", account2);

    // let ser = serde_binprot::to_vec(&account).unwrap();

    // println!("rust_random_account end");

    ser
}

// pub struct Account {
//     pub public_key: CompressedPubKey,         // Public_key.Compressed.t
//     pub token_id: TokenId,                    // Token_id.t
//     pub token_permissions: TokenPermissions,  // Token_permissions.t
//     pub token_symbol: TokenSymbol,            // Token_symbol.t
//     pub balance: Balance,                     // Balance.t
//     pub nonce: Nonce,                         // Nonce.t
//     pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
//     pub delegate: Option<CompressedPubKey>,   // Public_key.Compressed.t option
//     pub voting_for: VotingFor,                // State_hash.t
//     pub timing: Timing,                       // Timing.t
//     pub permissions: Permissions<AuthRequired>, // Permissions.t
//     pub zkapp: Option<ZkAppAccount>,          // Zkapp_account.t
//     pub zkapp_uri: String,                    // string
// }

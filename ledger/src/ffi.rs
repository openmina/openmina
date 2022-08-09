use ocaml_interop::{ocaml_export, OCaml, OCamlBytes, OCamlRef};

use crate::account::Account;

ocaml_export! {
    fn rust_add_account(
        rt,
        account: OCamlRef<OCamlBytes>,
    ) {
        let account_ref = rt.get(account);
        let account = account_ref.as_bytes();

        let account: Account = serde_binprot::from_slice(account).unwrap();

        println!("account={:?}", account);
        println!("account_hash={:?}", account.hash());

        OCaml::unit()
    }
}

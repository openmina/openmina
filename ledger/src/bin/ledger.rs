use mina_tree::*;

fn main() {
    for naccounts in [1_000, 10_000, 120_000] {
        println!("{:?} accounts wasmer", naccounts);

        let now = std::time::Instant::now();

        let mut db = Database::<V2>::create(20);

        let accounts = (0..naccounts).map(|_| Account::rand()).collect::<Vec<_>>();

        for (index, mut account) in accounts.into_iter().enumerate() {
            account.token_id = TokenId::from(index as u64);
            let id = account.id();
            db.get_or_create_account(id, account).unwrap();
        }

        println!("generate random accounts {:?}", now.elapsed());
        let now = std::time::Instant::now();

        assert_eq!(db.num_accounts(), naccounts as usize);

        db.merkle_root();

        println!("compute merkle root {:?}", now.elapsed());
    }
}

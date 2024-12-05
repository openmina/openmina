#![cfg_attr(feature = "nightly", feature(coverage_attribute))]
#![cfg_attr(feature = "nightly", feature(stmt_expr_attributes))]

#[cfg(feature = "nightly")]
pub mod transaction_fuzzer {
    pub mod context;
    pub mod coverage;
    pub mod generator;
    pub mod invariants;
    pub mod mutator;

    use binprot::{
        macros::{BinProtRead, BinProtWrite},
        BinProtRead, BinProtSize, BinProtWrite,
    };
    use context::{ApplyTxResult, FuzzerCtx, FuzzerCtxBuilder};
    use coverage::{
        cov::{Cov, FileCounters},
        reports::CoverageReport,
        stats::Stats,
    };
    use ledger::{
        scan_state::transaction_logic::UserCommand, sparse_ledger::LedgerIntf, Account, BaseLedger,
    };
    use mina_hasher::Fp;
    use mina_p2p_messages::bigint::BigInt;
    use openmina_core::constants::ConstraintConstantsUnversioned;
    use std::io::{Read, Write};
    use std::{
        env,
        process::{ChildStdin, ChildStdout},
    };

    #[coverage(off)]
    pub fn deserialize<T: BinProtRead, R: Read + ?Sized>(r: &mut R) -> T {
        let mut prefix_buf = [0u8; 4];
        r.read_exact(&mut prefix_buf).unwrap();
        // The OCaml process sends a len header for the binprot data, it seems we don't really need it but we must read it.
        let _prefix_len = u32::from_be_bytes(prefix_buf);
        T::binprot_read(r).unwrap()
    }

    #[coverage(off)]
    pub fn serialize<T: BinProtWrite, W: Write>(obj: &T, w: &mut W) {
        let size = obj.binprot_size() as u32;
        let prefix_buf: [u8; 4] = size.to_be_bytes();
        // The OCaml process expects a len header before the binprot data.
        w.write_all(prefix_buf.as_slice()).unwrap();
        obj.binprot_write(w).unwrap();
        w.flush().unwrap();
    }

    pub struct CoverageStats {
        cov: Cov,
        file_counters: Vec<FileCounters>,
        pub rust: Option<Stats>,
    }

    impl Default for CoverageStats {
        #[coverage(off)]
        fn default() -> Self {
            let mut cov = Cov::new();
            let file_counters = cov.get_file_counters();
            Self {
                cov,
                file_counters,
                rust: None,
            }
        }
    }

    impl CoverageStats {
        #[coverage(off)]
        pub fn new() -> Self {
            Self::default()
        }

        #[coverage(off)]
        pub fn update_rust(&mut self) -> bool {
            let rust_cov_stats = Stats::from_file_counters(&self.file_counters);
            let coverage_increased = self.rust.is_none()
                || rust_cov_stats.has_coverage_increased(self.rust.as_ref().unwrap());

            if coverage_increased {
                let llvm_dump = self.cov.dump();
                let report_rust = CoverageReport::from_llvm_dump(&llvm_dump);
                //println!("{}", report_rust);
                println!("Saving coverage report (Rust)");
                report_rust.write_files("rust".to_string());
            }

            self.rust = Some(rust_cov_stats);
            coverage_increased
        }

        #[coverage(off)]
        pub fn print(&self) {
            if let Some(stats) = &self.rust {
                println!(
                    "=== COV Rust ===\n{}",
                    stats
                        .filter_path(".cargo/") // unwanted files
                        .filter_path(".rustup/")
                        .filter_path("mina-p2p-messages/")
                        .filter_path("core/")
                        .filter_path("proofs/")
                );
            }
        }
    }

    #[derive(BinProtWrite, Debug)]
    enum Action {
        SetConstraintConstants(ConstraintConstantsUnversioned),
        SetInitialAccounts(Vec<Account>),
        GetAccounts,
        ApplyTx(UserCommand),
        #[allow(dead_code)]
        Exit,
    }

    #[derive(BinProtRead, Debug)]
    enum ActionOutput {
        ConstraintConstantsSet,
        InitialAccountsSet(BigInt),
        Accounts(Vec<Account>),
        TxApplied(ApplyTxResult),
        ExitAck,
    }

    #[coverage(off)]
    fn ocaml_set_initial_accounts(
        ctx: &mut FuzzerCtx,
        stdin: &mut ChildStdin,
        stdout: &mut ChildStdout,
    ) -> Fp {
        let action = Action::SetInitialAccounts(ctx.get_ledger_accounts());
        serialize(&action, stdin);
        let output: ActionOutput = deserialize(stdout);
        let ocaml_ledger_root_hash = match output {
            ActionOutput::InitialAccountsSet(root_hash) => root_hash,
            _ => panic!("Expected InitialAccountsSet"),
        };
        let rust_ledger_root_hash = ctx.get_ledger_root();
        assert!(ocaml_ledger_root_hash == rust_ledger_root_hash.into());
        rust_ledger_root_hash
    }

    #[coverage(off)]
    fn ocaml_get_accounts(stdin: &mut ChildStdin, stdout: &mut ChildStdout) -> Vec<Account> {
        let action = Action::GetAccounts;
        serialize(&action, stdin);
        let output: ActionOutput = deserialize(stdout);

        match output {
            ActionOutput::Accounts(accounts) => accounts,
            _ => unreachable!(),
        }
    }

    #[coverage(off)]
    fn ocaml_apply_transaction(
        stdin: &mut ChildStdin,
        stdout: &mut ChildStdout,
        user_command: UserCommand,
    ) -> ApplyTxResult {
        let action = Action::ApplyTx(user_command);
        serialize(&action, stdin);
        let output: ActionOutput = deserialize(stdout);
        match output {
            ActionOutput::TxApplied(result) => result,
            _ => panic!("Expected TxApplied"),
        }
    }

    #[coverage(off)]
    fn ocaml_set_constraint_constants(
        ctx: &mut FuzzerCtx,
        stdin: &mut ChildStdin,
        stdout: &mut ChildStdout,
    ) {
        let action = Action::SetConstraintConstants((&ctx.constraint_constants).into());
        serialize(&action, stdin);
        let output: ActionOutput = deserialize(stdout);
        match output {
            ActionOutput::ConstraintConstantsSet => (),
            _ => panic!("Expected ConstraintConstantsSet"),
        }
    }

    #[coverage(off)]
    pub fn fuzz(
        stdin: &mut ChildStdin,
        stdout: &mut ChildStdout,
        break_on_invariant: bool,
        seed: u64,
        minimum_fee: u64,
    ) {
        *invariants::BREAK.write().unwrap() = break_on_invariant;
        let mut cov_stats = CoverageStats::new();
        let mut ctx = FuzzerCtxBuilder::new()
            .seed(seed)
            .minimum_fee(minimum_fee)
            .initial_accounts(10)
            .fuzzcases_path(env::var("FUZZCASES_PATH").unwrap_or("/tmp/".to_string()))
            .build();

        ocaml_set_constraint_constants(&mut ctx, stdin, stdout);
        ocaml_set_initial_accounts(&mut ctx, stdin, stdout);

        let mut fuzzer_made_progress = false;

        for iteration in 0.. {
            print!("Iteration {}\r", iteration);
            std::io::stdout().flush().unwrap();

            if (iteration % 10000) == 0 {
                if fuzzer_made_progress {
                    fuzzer_made_progress = false;
                    ctx.take_snapshot();
                } else {
                    ctx.restore_snapshot();
                    // Restore ledger in OCaml
                    ocaml_set_initial_accounts(&mut ctx, stdin, stdout);
                }
            }

            // Update coverage statistics every 1000 iterations
            if (iteration % 1000) == 0 {
                let update_rust_increased_coverage = cov_stats.update_rust();

                if update_rust_increased_coverage {
                    fuzzer_made_progress = true;
                    cov_stats.print();
                }
            }

            let user_command: UserCommand = ctx.random_user_command();
            let ocaml_apply_result = ocaml_apply_transaction(stdin, stdout, user_command.clone());
            let mut ledger = ctx.get_ledger_inner().make_child();

            // Apply transaction on the Rust side
            if let Err(error) =
                ctx.apply_transaction(&mut ledger, &user_command, &ocaml_apply_result)
            {
                println!("!!! {error}");

                let ocaml_accounts = ocaml_get_accounts(stdin, stdout);
                let rust_accounts = ledger.to_list();

                for ocaml_account in ocaml_accounts.iter() {
                    match rust_accounts.iter().find(
                        #[coverage(off)]
                        |account| account.public_key == ocaml_account.public_key,
                    ) {
                        Some(rust_account) => {
                            if rust_account != ocaml_account {
                                println!(
                                    "Content mismatch between OCaml and Rust account:\n{}",
                                    ctx.diagnostic(rust_account, ocaml_account)
                                );
                            }
                        }
                        None => {
                            println!(
                                "OCaml account not present in Rust ledger: {:?}",
                                ocaml_account
                            );
                        }
                    }
                }

                for rust_account in rust_accounts.iter() {
                    if !ocaml_accounts.iter().any(
                        #[coverage(off)]
                        |account| account.public_key == rust_account.public_key,
                    ) {
                        println!(
                            "Rust account not present in Ocaml ledger: {:?}",
                            rust_account
                        );
                    }
                }

                let bigint: num_bigint::BigUint = LedgerIntf::merkle_root(&mut ledger).into();
                ctx.save_fuzzcase(&user_command, &bigint.to_string());

                // Exiting due to inconsistent state
                std::process::exit(0);
            }
        }
    }

    #[coverage(off)]
    pub fn reproduce(stdin: &mut ChildStdin, stdout: &mut ChildStdout, fuzzcase: &String) {
        let mut ctx = FuzzerCtxBuilder::new().build();
        let user_command = ctx.load_fuzzcase(fuzzcase);

        ocaml_set_constraint_constants(&mut ctx, stdin, stdout);
        ocaml_set_initial_accounts(&mut ctx, stdin, stdout);

        let mut ledger = ctx.get_ledger_inner().make_child();
        let ocaml_apply_result = ocaml_apply_transaction(stdin, stdout, user_command.clone());
        let rust_apply_result =
            ctx.apply_transaction(&mut ledger, &user_command, &ocaml_apply_result);

        println!("apply_transaction: {:?}", rust_apply_result);
    }
}

fn main() {
    #[cfg(feature = "nightly")]
    {
        use std::process::{Command, Stdio};

        let matches = clap::Command::new("Transaction Fuzzer")
            .arg(
                clap::Arg::new("fuzzcase")
                    .short('f')
                    .long("fuzzcase")
                    .value_name("FILE"),
            )
            .arg(
                clap::Arg::new("seed")
                    .short('s')
                    .long("seed")
                    .default_value("42")
                    .value_parser(clap::value_parser!(u64)),
            )
            .get_matches();

        let mut child = Command::new(
            std::env::var("OCAML_TRANSACTION_FUZZER_PATH").unwrap_or_else(
                #[coverage(off)]
                |_| {
                    format!(
                        "{}/mina/_build/default/src/app/transaction_fuzzer/transaction_fuzzer.exe",
                        std::env::var("HOME").unwrap()
                    )
                },
            ),
        )
        .arg("execute")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start OCaml process");

        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        let stdout = child.stdout.as_mut().expect("Failed to open stdout");

        if let Some(fuzzcase) = matches.get_one::<String>("fuzzcase") {
            println!("Reproducing fuzzcase from file: {}", fuzzcase);
            transaction_fuzzer::reproduce(stdin, stdout, fuzzcase);
        } else {
            let Some(seed) = matches.get_one::<u64>("seed") else {
                unreachable!()
            };

            println!("Running the fuzzer with seed {seed}...");
            transaction_fuzzer::fuzz(stdin, stdout, true, *seed, 1000);
        }
    }
}

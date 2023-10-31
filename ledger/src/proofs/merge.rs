use std::{path::Path, str::FromStr};

use ark_ff::One;
use kimchi::verifier_index::VerifierIndex;
use mina_curves::pasta::{Fq, Vesta};
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    proofs::witness::transaction_snark::assert_equal_local_state,
    scan_state::{
        fee_excess::FeeExcess,
        pending_coinbase,
        scan_state::transaction_snark::{
            validate_ledgers_at_merge_checked, SokDigest, SokMessage, Statement, StatementLedgers,
        },
    },
    VerificationKey,
};

use super::witness::{
    Boolean, MessagesForNextStepProof, PlonkVerificationKeyEvals, Prover, Witness,
};

fn read_witnesses() -> std::io::Result<Vec<Fp>> {
    let f = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("rampup4")
            .join("fps_merge.txt"),
    )?;
    // let f = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("fps.txt"))?;

    let fps = f
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| Fp::from_str(s).unwrap())
        .collect::<Vec<_>>();

    // TODO: Implement [0..652]
    // Ok(fps.split_off(652))
    Ok(fps)
}

fn merge_main(
    statement: Statement<SokDigest>,
    proofs: &(v2::LedgerProofProdStableV2, v2::LedgerProofProdStableV2),
    w: &mut Witness<Fp>,
) -> (Statement<SokDigest>, Statement<SokDigest>) {
    let (s1, s2) = w.exists({
        let (p1, p2) = proofs;
        let (s1, s2) = (&p1.0.statement, &p2.0.statement);
        let s1: Statement<SokDigest> = s1.into();
        let s2: Statement<SokDigest> = s2.into();
        (s1, s2)
    });

    let _fee_excess = FeeExcess::combine_checked(&s1.fee_excess, &s2.fee_excess, w);

    pending_coinbase::Stack::check_merge(
        (
            &s1.source.pending_coinbase_stack,
            &s1.target.pending_coinbase_stack,
        ),
        (
            &s2.source.pending_coinbase_stack,
            &s2.target.pending_coinbase_stack,
        ),
        w,
    );

    let _supply_increase = {
        let s1 = s1.supply_increase.to_checked::<Fp>();
        let s2 = s2.supply_increase.to_checked::<Fp>();

        // Made by `value` call in `add`
        w.exists_no_check(s1.value());
        w.exists_no_check(s2.value());

        s1.add(&s2, w)
    };

    assert_equal_local_state(&statement.source.local_state, &s1.source.local_state, w);
    assert_equal_local_state(&statement.target.local_state, &s2.target.local_state, w);

    let _valid_ledger = validate_ledgers_at_merge_checked(
        &StatementLedgers::of_statement(&s1),
        &StatementLedgers::of_statement(&s2),
        w,
    );

    {
        // Only `Statement.fee_excess`, not `fee_excess`
        let FeeExcess {
            fee_excess_l,
            fee_excess_r,
            ..
        } = statement.fee_excess;
        w.exists_no_check(fee_excess_l.to_checked::<Fp>().value());
        w.exists_no_check(fee_excess_r.to_checked::<Fp>().value());

        // Only `Statement.supply_increase`, not `supply_increase`
        let supply_increase = statement.supply_increase;
        w.exists_no_check(supply_increase.to_checked::<Fp>().value());
    }

    (s1, s2)
}

struct PreviousProofStatement<'a> {
    public_input: &'a Statement<SokDigest>,
    proof: &'a v2::LedgerProofProdStableV2,
    proof_must_verify: Boolean,
}

struct InductiveRule<'a> {
    previous_proof_statements: [PreviousProofStatement<'a>; 2],
    public_output: (),
    auxiliary_output: (),
}

fn dlog_plonk_index(wrap_prover: &Prover<Fq>) -> PlonkVerificationKeyEvals<Fp> {
    // TODO: Dedup `crate::PlonkVerificationKeyEvals` and `PlonkVerificationKeyEvals`
    let v =
        crate::PlonkVerificationKeyEvals::from(wrap_prover.index.verifier_index.as_ref().unwrap());
    PlonkVerificationKeyEvals::from(v)
}

fn expand_proof(
    dlog_vk: &VerifierIndex<Vesta>,
    app_state: &Statement<SokDigest>,
    t: &v2::LedgerProofProdStableV2,
    tag: (),
    must_verify: Boolean,
) {
    MessagesForNextStepProof {
        app_state,
        dlog_plonk_index: todo!(),
        challenge_polynomial_commitments: todo!(),
        old_bulletproof_challenges: todo!(),
    };
    // t.proof.statement.messages_for_next_step_proof;

    // t.proof.statement.proof_state.deferred_values.plonk;

    // let t =
    //   { t with
    //     statement =
    //       { t.statement with
    //         messages_for_next_step_proof =
    //           { t.statement.messages_for_next_step_proof with app_state }
    //       }
    //   }
    // in
    // let proof = Wrap_wire_proof.to_kimchi_proof t.proof in
    // let data = Types_map.lookup_basic tag in
    // let plonk0 = t.statement.proof_state.deferred_values.plonk in
    // let plonk =
}

pub fn generate_merge_proof(
    statement: &v2::MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    proofs: &(v2::LedgerProofProdStableV2, v2::LedgerProofProdStableV2),
    message: &SokMessage,
    step_prover: &Prover<Fp>,
    wrap_prover: &Prover<Fq>,
    w: &mut Witness<Fp>,
) {
    w.ocaml_aux = read_witnesses().unwrap();

    let statement: Statement<()> = statement.into();
    let sok_digest = message.digest();
    let statement_with_sok = statement.with_digest(sok_digest);

    w.exists(&statement_with_sok);

    let (s1, s2) = merge_main(statement_with_sok, proofs, w);
    let (p1, p2) = proofs;

    let rule = InductiveRule {
        previous_proof_statements: [
            PreviousProofStatement {
                public_input: &s1,
                proof: p1,
                proof_must_verify: Boolean::True,
            },
            PreviousProofStatement {
                public_input: &s2,
                proof: p2,
                proof_must_verify: Boolean::True,
            },
        ],
        public_output: (),
        auxiliary_output: (),
    };

    let dlog_plonk_index = w.exists(dlog_plonk_index(wrap_prover));

    dbg!(w.aux.len() + w.primary.capacity());
    dbg!(w.ocaml_aux.len());
}

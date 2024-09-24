use std::rc::Rc;

use crate::proofs::{
    constants::{make_step_transaction_data, StepMergeProof},
    step::{step, StepParams},
    util::sha256_sum,
    wrap::{wrap, WrapParams},
};
use ark_ff::fields::arithmetic::InvalidBigInt;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    proofs::transaction::transaction_snark::assert_equal_local_state,
    scan_state::{
        fee_excess::FeeExcess,
        pending_coinbase,
        scan_state::transaction_snark::{
            validate_ledgers_at_merge_checked, SokDigest, SokMessage, Statement, StatementLedgers,
        },
    },
};

use super::{
    constants::WrapMergeProof,
    field::{Boolean, CircuitVar, FieldWitness},
    public_input::plonk_checks::PlonkMinimal,
    step::{
        extract_recursion_challenges, InductiveRule, OptFlag, PreviousProofStatement, StepProof,
    },
    transaction::{PlonkVerificationKeyEvals, ProofError, Prover},
    util::two_u64_to_field,
    witness::Witness,
    wrap::WrapProof,
};

fn merge_main(
    statement: &Statement<SokDigest>,
    proofs: &[v2::LedgerProofProdStableV2; 2],
    w: &mut Witness<Fp>,
) -> Result<(Statement<SokDigest>, Statement<SokDigest>), InvalidBigInt> {
    let (s1, s2) = w.exists({
        let [p1, p2] = proofs;
        let (s1, s2) = (&p1.0.statement, &p2.0.statement);
        let s1: Statement<SokDigest> = s1.try_into()?;
        let s2: Statement<SokDigest> = s2.try_into()?;
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
        fee_excess_l.to_checked::<Fp>().value(w);
        fee_excess_r.to_checked::<Fp>().value(w);

        // Only `Statement.supply_increase`, not `supply_increase`
        let supply_increase = statement.supply_increase;
        supply_increase.to_checked::<Fp>().value(w);
    }

    Ok((s1, s2))
}

pub fn dlog_plonk_index(wrap_prover: &Prover<Fq>) -> PlonkVerificationKeyEvals<Fp> {
    PlonkVerificationKeyEvals::from(&**wrap_prover.index.verifier_index.as_ref().unwrap())
}

impl From<&v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags> for crate::proofs::step::FeatureFlags::<bool> {
    fn from(value: &v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags) -> Self {
        let v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags {
            range_check0,
            range_check1,
            foreign_field_add,
            foreign_field_mul,
            xor,
            rot,
            lookup,
            runtime_tables,
        } = value;

        Self {
            range_check0: *range_check0,
            range_check1: *range_check1,
            foreign_field_add: *foreign_field_add,
            foreign_field_mul: *foreign_field_mul,
            xor: *xor,
            rot: *rot,
            lookup: *lookup,
            runtime_tables: *runtime_tables,
        }
    }
}

impl From<&crate::proofs::step::FeatureFlags::<bool>> for v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags {
    fn from(value: &crate::proofs::step::FeatureFlags::<bool>) -> Self {
        let crate::proofs::step::FeatureFlags::<bool> {
            range_check0,
            range_check1,
            foreign_field_add,
            foreign_field_mul,
            xor,
            rot,
            lookup,
            runtime_tables,
        } = value;

        Self {
            range_check0: *range_check0,
            range_check1: *range_check1,
            foreign_field_add: *foreign_field_add,
            foreign_field_mul: *foreign_field_mul,
            xor: *xor,
            rot: *rot,
            lookup: *lookup,
            runtime_tables: *runtime_tables,
        }
    }
}

impl<F: FieldWitness>
    TryFrom<&v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk>
    for PlonkMinimal<F>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk,
    ) -> Result<Self, Self::Error> {
        let v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk {
            alpha,
            beta,
            gamma,
            zeta,
            joint_combiner,
            feature_flags,
        } = value;

        let to_bytes = |v: &v2::LimbVectorConstantHex64StableV1| v.as_u64();

        let alpha_bytes = alpha.inner.each_ref().map(to_bytes);
        let beta_bytes = beta.each_ref().map(to_bytes);
        let gamma_bytes = gamma.each_ref().map(to_bytes);
        let zeta_bytes = zeta.inner.each_ref().map(to_bytes);

        Ok(PlonkMinimal::<F, 2> {
            alpha: two_u64_to_field(&alpha_bytes),
            beta: two_u64_to_field(&beta_bytes),
            gamma: two_u64_to_field(&gamma_bytes),
            zeta: two_u64_to_field(&zeta_bytes),
            joint_combiner: joint_combiner
                .as_ref()
                .map(|f| two_u64_to_field(&f.inner.each_ref().map(to_bytes))),
            alpha_bytes,
            beta_bytes,
            gamma_bytes,
            zeta_bytes,
            joint_combiner_bytes: joint_combiner
                .as_ref()
                .map(|f| f.inner.each_ref().map(to_bytes)),
            feature_flags: feature_flags.into(),
        })
    }
}

pub struct MergeParams<'a> {
    pub statement: Statement<()>,
    pub proofs: &'a [v2::LedgerProofProdStableV2; 2],
    pub message: &'a SokMessage,
    pub step_prover: &'a Prover<Fp>,
    pub wrap_prover: &'a Prover<Fq>,
    /// When set to `true`, `generate_block_proof` will not create a proof, but only
    /// verify constraints in the step witnesses
    pub only_verify_constraints: bool,
    /// For debugging only
    pub expected_step_proof: Option<&'static str>,
    /// For debugging only
    pub ocaml_wrap_witness: Option<Vec<Fq>>,
}

const MERGE_N_PREVIOUS_PROOFS: usize = 2;

pub(super) fn generate_merge_proof(
    params: MergeParams,
    w: &mut Witness<Fp>,
) -> Result<WrapProof, ProofError> {
    let MergeParams {
        statement,
        proofs,
        message,
        step_prover,
        wrap_prover,
        only_verify_constraints,
        expected_step_proof,
        ocaml_wrap_witness,
    } = params;

    let sok_digest = message.digest();
    let statement_with_sok = Rc::new(statement.with_digest(sok_digest));

    w.exists(&*statement_with_sok);

    let (s1, s2) = merge_main(&statement_with_sok, proofs, w)?;

    let [p1, p2]: [&v2::PicklesProofProofsVerified2ReprStableV2; 2] = {
        let [p1, p2] = proofs;
        [&p1.0.proof, &p2.0.proof]
    };

    let prev_challenge_polynomial_commitments = extract_recursion_challenges(&[p1, p2])?;

    let rule = InductiveRule {
        previous_proof_statements: [
            PreviousProofStatement {
                public_input: Rc::new(s1),
                proof: p1,
                proof_must_verify: CircuitVar::Constant(Boolean::True),
            },
            PreviousProofStatement {
                public_input: Rc::new(s2),
                proof: p2,
                proof_must_verify: CircuitVar::Constant(Boolean::True),
            },
        ],
        public_output: (),
        auxiliary_output: (),
    };

    let dlog_plonk_index = dlog_plonk_index(wrap_prover);
    let dlog_plonk_index_cvar = dlog_plonk_index.to_cvar(CircuitVar::Var);
    let verifier_index = &**wrap_prover.index.verifier_index.as_ref().unwrap();

    let tx_data = make_step_transaction_data(&dlog_plonk_index_cvar);
    let for_step_datas = [&tx_data, &tx_data];

    let indexes = [
        (verifier_index, &dlog_plonk_index_cvar),
        (verifier_index, &dlog_plonk_index_cvar),
    ];

    let StepProof {
        statement,
        prev_evals,
        proof_with_public: proof,
    } = step::<StepMergeProof, MERGE_N_PREVIOUS_PROOFS>(
        StepParams {
            app_state: Rc::clone(&statement_with_sok) as _,
            rule,
            for_step_datas,
            indexes,
            wrap_prover,
            prev_challenge_polynomial_commitments,
            step_prover,
            hack_feature_flags: OptFlag::No,
            only_verify_constraints,
        },
        w,
    )?;

    if let Some(expected) = expected_step_proof {
        let proof_json = serde_json::to_vec(&proof.proof).unwrap();
        assert_eq!(sha256_sum(&proof_json), expected);
    };

    let mut w = Witness::new::<WrapMergeProof>();

    if let Some(ocaml_aux) = ocaml_wrap_witness {
        w.ocaml_aux = ocaml_aux;
    };

    wrap::<WrapMergeProof>(
        WrapParams {
            app_state: statement_with_sok,
            proof_with_public: &proof,
            step_statement: statement,
            prev_evals: &prev_evals,
            dlog_plonk_index: &dlog_plonk_index,
            step_prover_index: &step_prover.index,
            wrap_prover,
        },
        &mut w,
    )
}

use std::rc::Rc;

use ark_ff::fields::arithmetic::InvalidBigInt;
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use ark_serialize::Write;
use itertools::Itertools;
use poly_commitment::srs::SRS;

use crate::{
    proofs::{
        accumulator_check,
        step::{expand_deferred, StatementProofState},
        unfinalized::AllEvals,
        verifiers::make_zkapp_verifier_index,
        wrap::Domain,
        BACKEND_TICK_ROUNDS_N,
    },
    scan_state::{
        protocol_state::MinaHash,
        scan_state::transaction_snark::{SokDigest, Statement},
        transaction_logic::{local_state::LazyValue, zkapp_statement::ZkappStatement},
    },
    VerificationKey,
};

use super::{
    block::ProtocolState,
    field::FieldWitness,
    public_input::plonk_checks::make_shifts,
    step::{step_verifier::PlonkDomain, ExpandDeferredParams},
    to_field_elements::ToFieldElements,
    transaction::{InnerCurve, PlonkVerificationKeyEvals},
    util::{extract_bulletproof, extract_polynomial_commitment, two_u64_to_field},
    wrap::expand_feature_flags,
    ProverProof, VerifierIndex,
};
use kimchi::{
    circuits::{expr::RowOffset, wires::PERMUTS},
    error::VerifyError,
    mina_curves::pasta::Pallas,
    proof::{PointEvaluations, ProofEvaluations},
};
use mina_curves::pasta::{Fq, Vesta};
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    binprot::BinProtWrite,
    v2::{
        self, CompositionTypesDigestConstantStableV1, MinaBlockHeaderStableV2,
        PicklesProofProofsVerified2ReprStableV2,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags,
        TransactionSnarkProofStableV2,
    },
};

use super::prover::make_padded_proof_from_p2p;

use super::public_input::{
    messages::{MessagesForNextStepProof, MessagesForNextWrapProof},
    plonk_checks::{PlonkMinimal, ScalarsEnv},
    prepared_statement::{DeferredValues, PreparedStatement, ProofState},
};

#[cfg(target_family = "wasm")]
#[cfg(test)]
mod wasm {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
}

fn validate_feature_flags(
    feature_flags: &PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags,
    evals: &PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
) -> bool {
    let PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
        w: _,
        coefficients: _,
        z: _,
        s: _,
        generic_selector: _,
        poseidon_selector: _,
        complete_add_selector: _,
        mul_selector: _,
        emul_selector: _,
        endomul_scalar_selector: _,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = evals;

    fn enable_if<T>(x: &Option<T>, flag: bool) -> bool {
        x.is_some() == flag
    }

    let f = feature_flags;
    let range_check_lookup = f.range_check0 || f.range_check1 || f.rot;
    let lookups_per_row_4 = f.xor || range_check_lookup || f.foreign_field_mul;
    let lookups_per_row_3 = lookups_per_row_4 || f.lookup;
    let lookups_per_row_2 = lookups_per_row_3;

    [
        enable_if(range_check0_selector, f.range_check0),
        enable_if(range_check1_selector, f.range_check1),
        enable_if(foreign_field_add_selector, f.foreign_field_add),
        enable_if(foreign_field_mul_selector, f.foreign_field_mul),
        enable_if(xor_selector, f.xor),
        enable_if(rot_selector, f.rot),
        enable_if(lookup_aggregation, lookups_per_row_2),
        enable_if(lookup_table, lookups_per_row_2),
        lookup_sorted.iter().enumerate().fold(true, |acc, (i, x)| {
            let flag = match i {
                0..=2 => lookups_per_row_2,
                3 => lookups_per_row_3,
                4 => lookups_per_row_4,
                _ => panic!(),
            };
            acc && enable_if(x, flag)
        }),
        enable_if(runtime_lookup_table, f.runtime_tables),
        enable_if(runtime_lookup_table_selector, f.runtime_tables),
        enable_if(xor_lookup_selector, f.xor),
        enable_if(lookup_gate_lookup_selector, f.lookup),
        enable_if(range_check_lookup_selector, range_check_lookup),
        enable_if(foreign_field_mul_lookup_selector, f.foreign_field_mul),
    ]
    .iter()
    .all(|b| *b)
}

pub fn prev_evals_from_p2p<F: FieldWitness>(
    evals: &PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
) -> Result<ProofEvaluations<PointEvaluations<Vec<F>>>, InvalidBigInt> {
    let PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
        w,
        coefficients,
        z,
        s,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = evals;

    fn of<'a, F: FieldWitness, I: IntoIterator<Item = &'a BigInt>>(
        zeta: I,
        zeta_omega: I,
    ) -> Result<PointEvaluations<Vec<F>>, InvalidBigInt> {
        Ok(PointEvaluations {
            zeta: zeta
                .into_iter()
                .map(BigInt::to_field)
                .collect::<Result<_, _>>()?,
            zeta_omega: zeta_omega
                .into_iter()
                .map(BigInt::to_field)
                .collect::<Result<_, _>>()?,
        })
    }

    let of = |(zeta, zeta_omega): &(_, _)| -> Result<PointEvaluations<Vec<F>>, _> {
        of(zeta, zeta_omega)
    };
    let of_opt = |v: &Option<(_, _)>| match v.as_ref() {
        Some(v) => Ok(Some(of(v)?)),
        None => Ok(None),
    };

    Ok(ProofEvaluations {
        public: None,
        w: crate::try_array_into_with(w, of)?,
        z: of(z)?,
        s: crate::try_array_into_with(s, of)?,
        coefficients: crate::try_array_into_with(coefficients, of)?,
        generic_selector: of(generic_selector)?,
        poseidon_selector: of(poseidon_selector)?,
        complete_add_selector: of(complete_add_selector)?,
        mul_selector: of(mul_selector)?,
        emul_selector: of(emul_selector)?,
        endomul_scalar_selector: of(endomul_scalar_selector)?,
        range_check0_selector: of_opt(range_check0_selector)?,
        range_check1_selector: of_opt(range_check1_selector)?,
        foreign_field_add_selector: of_opt(foreign_field_add_selector)?,
        foreign_field_mul_selector: of_opt(foreign_field_mul_selector)?,
        xor_selector: of_opt(xor_selector)?,
        rot_selector: of_opt(rot_selector)?,
        lookup_aggregation: of_opt(lookup_aggregation)?,
        lookup_table: of_opt(lookup_table)?,
        lookup_sorted: crate::try_array_into_with(lookup_sorted, of_opt)?,
        runtime_lookup_table: of_opt(runtime_lookup_table)?,
        runtime_lookup_table_selector: of_opt(runtime_lookup_table_selector)?,
        xor_lookup_selector: of_opt(xor_lookup_selector)?,
        lookup_gate_lookup_selector: of_opt(lookup_gate_lookup_selector)?,
        range_check_lookup_selector: of_opt(range_check_lookup_selector)?,
        foreign_field_mul_lookup_selector: of_opt(foreign_field_mul_lookup_selector)?,
    })
}

pub fn prev_evals_to_p2p(
    evals: &ProofEvaluations<PointEvaluations<Vec<Fp>>>,
) -> PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
    let ProofEvaluations {
        public: _,
        w,
        coefficients,
        z,
        s,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = evals;

    use mina_p2p_messages::pseq::PaddedSeq;

    let of = |PointEvaluations { zeta, zeta_omega }: &PointEvaluations<Vec<Fp>>| {
        (
            zeta.iter().map(Into::into).collect(),
            zeta_omega.iter().map(Into::into).collect(),
        )
    };

    let of_opt = |v: &Option<PointEvaluations<Vec<Fp>>>| v.as_ref().map(of);

    PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
        w: PaddedSeq(w.each_ref().map(of)),
        z: of(z),
        s: PaddedSeq(s.each_ref().map(of)),
        coefficients: PaddedSeq(coefficients.each_ref().map(of)),
        generic_selector: of(generic_selector),
        poseidon_selector: of(poseidon_selector),
        complete_add_selector: of(complete_add_selector),
        mul_selector: of(mul_selector),
        emul_selector: of(emul_selector),
        endomul_scalar_selector: of(endomul_scalar_selector),
        range_check0_selector: of_opt(range_check0_selector),
        range_check1_selector: of_opt(range_check1_selector),
        foreign_field_add_selector: of_opt(foreign_field_add_selector),
        foreign_field_mul_selector: of_opt(foreign_field_mul_selector),
        xor_selector: of_opt(xor_selector),
        rot_selector: of_opt(rot_selector),
        lookup_aggregation: of_opt(lookup_aggregation),
        lookup_table: of_opt(lookup_table),
        lookup_sorted: PaddedSeq(lookup_sorted.each_ref().map(of_opt)),
        runtime_lookup_table: of_opt(runtime_lookup_table),
        runtime_lookup_table_selector: of_opt(runtime_lookup_table_selector),
        xor_lookup_selector: of_opt(xor_lookup_selector),
        lookup_gate_lookup_selector: of_opt(lookup_gate_lookup_selector),
        range_check_lookup_selector: of_opt(range_check_lookup_selector),
        foreign_field_mul_lookup_selector: of_opt(foreign_field_mul_lookup_selector),
    }
}

struct LimitedDomain<F: FieldWitness> {
    domain: Radix2EvaluationDomain<F>,
    shifts: kimchi::circuits::polynomials::permutation::Shifts<F>,
}

impl<F: FieldWitness> PlonkDomain<F> for LimitedDomain<F> {
    fn vanishing_polynomial(&self, _x: F, _w: &mut super::witness::Witness<F>) -> F {
        unimplemented!() // Unused during proof verification
    }
    fn generator(&self) -> F {
        self.domain.group_gen
    }
    fn shifts(&self) -> &[F; PERMUTS] {
        self.shifts.shifts()
    }
    fn log2_size(&self) -> u64 {
        unimplemented!() // Unused during proof verification
    }
}

// TODO: `domain_log2` and `srs_length_log2` might be the same here ? Remove one or the other
pub fn make_scalars_env<F: FieldWitness, const NLIMB: usize>(
    minimal: &PlonkMinimal<F, NLIMB>,
    domain_log2: u8,
    srs_length_log2: u64,
    zk_rows: u64,
) -> ScalarsEnv<F> {
    let domain: Radix2EvaluationDomain<F> =
        Radix2EvaluationDomain::new(1 << domain_log2 as u64).unwrap();

    let zeta_to_n_minus_1 = domain.evaluate_vanishing_polynomial(minimal.zeta);

    let (
        omega_to_zk_minus_1,
        omega_to_zk,
        omega_to_intermediate_powers,
        omega_to_zk_plus_1,
        omega_to_minus_1,
    ) = {
        let gen = domain.group_gen;
        let omega_to_minus_1 = F::one() / gen;
        let omega_to_minus_2 = omega_to_minus_1.square();
        let (omega_to_intermediate_powers, omega_to_zk_plus_1) = {
            let mut next_term = omega_to_minus_2;
            let omega_to_intermediate_powers = (0..(zk_rows.checked_sub(3).unwrap()))
                .map(|_| {
                    let term = next_term;
                    next_term = term * omega_to_minus_1;
                    term
                })
                .collect::<Vec<_>>();
            (omega_to_intermediate_powers, next_term)
        };
        let omega_to_zk = omega_to_zk_plus_1 * omega_to_minus_1;
        let omega_to_zk_minus_1 = move || omega_to_zk * omega_to_minus_1;

        (
            omega_to_zk_minus_1,
            omega_to_zk,
            omega_to_intermediate_powers,
            omega_to_zk_plus_1,
            omega_to_minus_1,
        )
    };

    let zk_polynomial = (minimal.zeta - omega_to_minus_1)
        * (minimal.zeta - omega_to_zk_plus_1)
        * (minimal.zeta - omega_to_zk);

    let shifts = make_shifts(&domain);
    let domain = Rc::new(LimitedDomain { domain, shifts });

    let vanishes_on_zero_knowledge_and_previous_rows = match minimal.joint_combiner {
        None => F::one(),
        Some(_) => omega_to_intermediate_powers.iter().fold(
            // init
            zk_polynomial * (minimal.zeta - omega_to_zk_minus_1()),
            // f
            |acc, omega_pow| acc * (minimal.zeta - omega_pow),
        ),
    };

    let zeta_clone = minimal.zeta;
    let zeta_to_srs_length =
        LazyValue::make(move |_| (0..srs_length_log2).fold(zeta_clone, |acc, _| acc * acc));

    let feature_flags = minimal
        .joint_combiner
        .map(|_| expand_feature_flags::<F>(&minimal.feature_flags.to_boolean()));

    let unnormalized_lagrange_basis = match minimal.joint_combiner {
        None => None,
        Some(_) => {
            use crate::proofs::witness::Witness;

            let zeta = minimal.zeta;
            let generator = domain.generator();
            let omega_to_zk_minus_1_clone = omega_to_zk_minus_1();
            let fun: Box<dyn Fn(RowOffset, &mut Witness<F>) -> F> =
                Box::new(move |i: RowOffset, w: &mut Witness<F>| {
                    let w_to_i = match (i.zk_rows, i.offset) {
                        (false, 0) => F::one(),
                        (false, 1) => generator,
                        (false, -1) => omega_to_minus_1,
                        (false, -2) => omega_to_zk_plus_1,
                        (false, -3) | (true, 0) => omega_to_zk,
                        (true, -1) => omega_to_zk_minus_1_clone,
                        _ => todo!(),
                    };
                    crate::proofs::field::field::div_by_inv(zeta_to_n_minus_1, zeta - w_to_i, w)
                });
            Some(fun)
        }
    };

    ScalarsEnv {
        zk_polynomial,
        zeta_to_n_minus_1,
        srs_length_log2,
        domain,
        omega_to_minus_zk_rows: omega_to_zk,
        feature_flags,
        unnormalized_lagrange_basis,
        vanishes_on_zero_knowledge_and_previous_rows,
        zeta_to_srs_length,
    }
}

fn get_message_for_next_step_proof<'a, AppState>(
    messages_for_next_step_proof: &PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
    commitments: &'a PlonkVerificationKeyEvals<Fp>,
    app_state: &'a AppState,
) -> Result<MessagesForNextStepProof<'a, AppState>, InvalidBigInt>
where
    AppState: ToFieldElements<Fp>,
{
    let PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
        app_state: _, // unused
        challenge_polynomial_commitments,
        old_bulletproof_challenges,
    } = messages_for_next_step_proof;

    let challenge_polynomial_commitments: Vec<InnerCurve<Fp>> =
        extract_polynomial_commitment(challenge_polynomial_commitments)?;
    let old_bulletproof_challenges: Vec<[Fp; 16]> = extract_bulletproof(old_bulletproof_challenges);
    let dlog_plonk_index = commitments;

    Ok(MessagesForNextStepProof {
        app_state,
        dlog_plonk_index,
        challenge_polynomial_commitments,
        old_bulletproof_challenges,
    })
}

fn get_message_for_next_wrap_proof(
    PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
        challenge_polynomial_commitment,
        old_bulletproof_challenges,
    }: &PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
) -> Result<MessagesForNextWrapProof, InvalidBigInt> {
    let challenge_polynomial_commitments: Vec<InnerCurve<Fq>> =
        extract_polynomial_commitment(&[challenge_polynomial_commitment.clone()])?;

    let old_bulletproof_challenges: Vec<[Fq; 15]> = extract_bulletproof(&[
        old_bulletproof_challenges[0].0.clone(),
        old_bulletproof_challenges[1].0.clone(),
    ]);

    Ok(MessagesForNextWrapProof {
        challenge_polynomial_commitment: challenge_polynomial_commitments[0].clone(),
        old_bulletproof_challenges,
    })
}

fn get_prepared_statement<AppState>(
    message_for_next_step_proof: &MessagesForNextStepProof<AppState>,
    message_for_next_wrap_proof: &MessagesForNextWrapProof,
    deferred_values: DeferredValues<Fp>,
    sponge_digest_before_evaluations: &CompositionTypesDigestConstantStableV1,
) -> PreparedStatement
where
    AppState: ToFieldElements<Fp>,
{
    let digest = sponge_digest_before_evaluations;
    let sponge_digest_before_evaluations: [u64; 4] = digest.each_ref().map(|v| v.as_u64());

    PreparedStatement {
        proof_state: ProofState {
            deferred_values,
            sponge_digest_before_evaluations,
            messages_for_next_wrap_proof: message_for_next_wrap_proof.hash(),
        },
        messages_for_next_step_proof: message_for_next_step_proof.hash(),
    }
}

fn verify_with(
    verifier_index: &VerifierIndex<Fq>,
    proof: &ProverProof<Fq>,
    public_input: &[Fq],
) -> Result<(), VerifyError> {
    use kimchi::groupmap::GroupMap;
    use kimchi::mina_curves::pasta::PallasParameters;
    use mina_poseidon::sponge::{DefaultFqSponge, DefaultFrSponge};
    use poly_commitment::evaluation_proof::OpeningProof;

    type SpongeParams = mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    type EFqSponge = DefaultFqSponge<PallasParameters, SpongeParams>;
    type EFrSponge = DefaultFrSponge<Fq, SpongeParams>;

    let group_map = GroupMap::<Fp>::setup();

    kimchi::verifier::verify::<Pallas, EFqSponge, EFrSponge, OpeningProof<Pallas>>(
        &group_map,
        verifier_index,
        proof,
        public_input,
    )
}

pub struct VerificationContext<'a> {
    pub verifier_index: &'a VerifierIndex<Fq>,
    pub proof: &'a ProverProof<Fq>,
    pub public_input: &'a [Fq],
}

fn batch_verify(proofs: &[VerificationContext]) -> Result<(), VerifyError> {
    use kimchi::groupmap::GroupMap;
    use kimchi::mina_curves::pasta::PallasParameters;
    use kimchi::verifier::Context;
    use mina_poseidon::sponge::{DefaultFqSponge, DefaultFrSponge};
    use poly_commitment::evaluation_proof::OpeningProof;

    type SpongeParams = mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    type EFqSponge = DefaultFqSponge<PallasParameters, SpongeParams>;
    type EFrSponge = DefaultFrSponge<Fq, SpongeParams>;

    let group_map = GroupMap::<Fp>::setup();
    let proofs = proofs
        .iter()
        .map(|p| Context {
            verifier_index: p.verifier_index,
            proof: p.proof,
            public_input: p.public_input,
        })
        .collect_vec();

    kimchi::verifier::batch_verify::<Pallas, EFqSponge, EFrSponge, OpeningProof<Pallas>>(
        &group_map, &proofs,
    )
}

fn run_checks(
    proof: &PicklesProofProofsVerified2ReprStableV2,
    verifier_index: &VerifierIndex<Fq>,
) -> bool {
    let mut errors: Vec<String> = vec![];
    let mut checks = |condition: bool, s: &str| {
        if !condition {
            errors.push(s.to_string())
        }
    };

    let non_chunking = {
        let PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
            w,
            coefficients,
            z,
            s,
            generic_selector,
            poseidon_selector,
            complete_add_selector,
            mul_selector,
            emul_selector,
            endomul_scalar_selector,
            range_check0_selector,
            range_check1_selector,
            foreign_field_add_selector,
            foreign_field_mul_selector,
            xor_selector,
            rot_selector,
            lookup_aggregation,
            lookup_table,
            lookup_sorted,
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        } = &proof.prev_evals.evals.evals;

        let mut iter = w
            .iter()
            .chain(coefficients.iter())
            .chain([z])
            .chain(s.iter())
            .chain([
                generic_selector,
                poseidon_selector,
                complete_add_selector,
                mul_selector,
                emul_selector,
                endomul_scalar_selector,
            ])
            .chain(range_check0_selector.iter())
            .chain(range_check1_selector.iter())
            .chain(foreign_field_add_selector.iter())
            .chain(foreign_field_mul_selector.iter())
            .chain(xor_selector.iter())
            .chain(rot_selector.iter())
            .chain(lookup_aggregation.iter())
            .chain(lookup_table.iter())
            .chain(lookup_sorted.iter().flatten())
            .chain(runtime_lookup_table.iter())
            .chain(runtime_lookup_table_selector.iter())
            .chain(xor_lookup_selector.iter())
            .chain(lookup_gate_lookup_selector.iter())
            .chain(range_check_lookup_selector.iter())
            .chain(foreign_field_mul_lookup_selector.iter());

        iter.all(|(a, b)| a.len() <= 1 && b.len() <= 1)
    };

    checks(non_chunking, "only uses single chunks");

    checks(
        validate_feature_flags(
            &proof
                .statement
                .proof_state
                .deferred_values
                .plonk
                .feature_flags,
            &proof.prev_evals.evals.evals,
        ),
        "feature flags are consistent with evaluations",
    );

    let branch_data = &proof.statement.proof_state.deferred_values.branch_data;
    let step_domain: u8 = branch_data.domain_log2.as_u8();
    let step_domain = Domain::Pow2RootsOfUnity(step_domain as u64);

    checks(
        step_domain.log2_size() as usize <= BACKEND_TICK_ROUNDS_N,
        "domain size is small enough",
    );

    {
        // TODO: Don't use hardcoded values
        let all_possible_domains = [13, 14, 15];
        let [greatest_wrap_domain, _, least_wrap_domain] = all_possible_domains;

        let actual_wrap_domain = verifier_index.domain.log_size_of_group;
        checks(
            actual_wrap_domain <= least_wrap_domain,
            "invalid actual_wrap_domain (least_wrap_domain)",
        );
        checks(
            actual_wrap_domain >= greatest_wrap_domain,
            "invalid actual_wrap_domain (greatest_wrap_domain)",
        );
    }

    for e in &errors {
        eprintln!("{:?}", e);
    }

    errors.is_empty()
}

fn compute_deferred_values(
    proof: &PicklesProofProofsVerified2ReprStableV2,
) -> Result<DeferredValues<Fp>, InvalidBigInt> {
    let bulletproof_challenges: Vec<Fp> = proof
        .statement
        .proof_state
        .deferred_values
        .bulletproof_challenges
        .iter()
        .map(|chal| {
            let prechallenge = &chal.prechallenge.inner;
            let prechallenge: [u64; 2] = prechallenge.each_ref().map(|v| v.as_u64());
            two_u64_to_field(&prechallenge)
        })
        .collect();

    let deferred_values = {
        let old_bulletproof_challenges: Vec<[Fp; 16]> = proof
            .statement
            .messages_for_next_step_proof
            .old_bulletproof_challenges
            .iter()
            .map(|v| {
                v.0.clone()
                    .map(|v| two_u64_to_field(&v.prechallenge.inner.0.map(|v| v.as_u64())))
            })
            .collect();
        let proof_state: StatementProofState = (&proof.statement.proof_state).try_into()?;
        let evals: AllEvals<Fp> = (&proof.prev_evals).try_into()?;

        let zk_rows = 3;
        expand_deferred(ExpandDeferredParams {
            evals: &evals,
            old_bulletproof_challenges: &old_bulletproof_challenges,
            proof_state: &proof_state,
            zk_rows,
        })?
    };

    Ok(DeferredValues {
        bulletproof_challenges,
        ..deferred_values
    })
}

/// https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/pickles/verification_key.mli#L30
pub struct VK<'a> {
    pub commitments: PlonkVerificationKeyEvals<Fp>,
    pub index: &'a VerifierIndex<Fq>,
    pub data: (), // Unused in proof verification
}

pub fn verify_block(
    header: &MinaBlockHeaderStableV2,
    verifier_index: &VerifierIndex<Fq>,
    srs: &SRS<Vesta>,
) -> bool {
    let MinaBlockHeaderStableV2 {
        protocol_state,
        protocol_state_proof,
        ..
    } = &header;

    let vk = VK {
        commitments: PlonkVerificationKeyEvals::from(verifier_index),
        index: verifier_index,
        data: (),
    };

    let Ok(protocol_state) = ProtocolState::try_from(protocol_state) else {
        return false; // invalid bigint
    };
    let protocol_state_hash = MinaHash::hash(&protocol_state);

    let accum_check =
        accumulator_check::accumulator_check(srs, &[protocol_state_proof]).unwrap_or(false);
    let verified = verify_impl(&protocol_state_hash, protocol_state_proof, &vk).unwrap_or(false);

    accum_check && verified
}

pub fn verify_transaction<'a>(
    proofs: impl IntoIterator<Item = (&'a Statement<SokDigest>, &'a TransactionSnarkProofStableV2)>,
    verifier_index: &VerifierIndex<Fq>,
    srs: &SRS<Vesta>,
) -> bool {
    let vk = VK {
        commitments: PlonkVerificationKeyEvals::from(verifier_index),
        index: verifier_index,
        data: (),
    };

    let mut inputs: Vec<(
        &Statement<SokDigest>,
        &PicklesProofProofsVerified2ReprStableV2,
        &VK,
    )> = Vec::with_capacity(128);

    let mut accum_check_proofs: Vec<&PicklesProofProofsVerified2ReprStableV2> =
        Vec::with_capacity(128);

    proofs
        .into_iter()
        .for_each(|(statement, transaction_proof)| {
            accum_check_proofs.push(transaction_proof);
            inputs.push((statement, transaction_proof, &vk));
        });

    let accum_check =
        accumulator_check::accumulator_check(srs, &accum_check_proofs).unwrap_or(false);

    let verified = batch_verify_impl(inputs.as_slice()).unwrap_or(false);
    accum_check && verified
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/crypto/kimchi_bindings/stubs/src/pasta_fq_plonk_proof.rs#L116
pub fn verify_zkapp(
    verification_key: &VerificationKey,
    zkapp_statement: &ZkappStatement,
    sideloaded_proof: &PicklesProofProofsVerified2ReprStableV2,
    srs: &SRS<Vesta>,
) -> bool {
    let verifier_index = make_zkapp_verifier_index(verification_key);
    // https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/pickles/pickles.ml#LL260C1-L274C18
    let vk = VK {
        commitments: *verification_key.wrap_index.clone(),
        index: &verifier_index,
        data: (),
    };

    let accum_check =
        accumulator_check::accumulator_check(srs, &[sideloaded_proof]).unwrap_or(false);
    let verified = verify_impl(&zkapp_statement, sideloaded_proof, &vk).unwrap_or(false);

    let ok = accum_check && verified;

    eprintln!("verify_zkapp OK={:?}", ok);

    #[cfg(not(test))]
    if !ok {
        if let Err(e) = dump_zkapp_verification(verification_key, zkapp_statement, sideloaded_proof)
        {
            eprintln!("Failed to dump zkapp verification: {:?}", e);
        }
    }

    ok
}

fn verify_impl<AppState>(
    app_state: &AppState,
    proof: &PicklesProofProofsVerified2ReprStableV2,
    vk: &VK,
) -> Result<bool, InvalidBigInt>
where
    AppState: ToFieldElements<Fp>,
{
    let deferred_values = compute_deferred_values(proof)?;
    let checks = run_checks(proof, vk.index);

    let message_for_next_step_proof = get_message_for_next_step_proof(
        &proof.statement.messages_for_next_step_proof,
        &vk.commitments,
        app_state,
    )?;

    let message_for_next_wrap_proof =
        get_message_for_next_wrap_proof(&proof.statement.proof_state.messages_for_next_wrap_proof)?;

    let prepared_statement = get_prepared_statement(
        &message_for_next_step_proof,
        &message_for_next_wrap_proof,
        deferred_values,
        &proof.statement.proof_state.sponge_digest_before_evaluations,
    );

    let npublic_input = vk.index.public;
    let public_inputs = prepared_statement.to_public_input(npublic_input)?;
    let proof = make_padded_proof_from_p2p(proof)?;

    let result = verify_with(vk.index, &proof, &public_inputs);

    if let Err(e) = result {
        eprintln!("verify error={:?}", e);
    };

    Ok(result.is_ok() && checks)
}

fn batch_verify_impl<AppState>(
    proofs: &[(&AppState, &PicklesProofProofsVerified2ReprStableV2, &VK)],
) -> Result<bool, InvalidBigInt>
where
    AppState: ToFieldElements<Fp>,
{
    let mut verification_contexts = Vec::with_capacity(proofs.len());
    let mut checks = true;

    for (app_state, proof, vk) in proofs {
        let deferred_values = compute_deferred_values(proof)?;
        checks = checks && run_checks(proof, vk.index);

        let message_for_next_step_proof = get_message_for_next_step_proof(
            &proof.statement.messages_for_next_step_proof,
            &vk.commitments,
            app_state,
        )?;

        let message_for_next_wrap_proof = get_message_for_next_wrap_proof(
            &proof.statement.proof_state.messages_for_next_wrap_proof,
        )?;

        let prepared_statement = get_prepared_statement(
            &message_for_next_step_proof,
            &message_for_next_wrap_proof,
            deferred_values,
            &proof.statement.proof_state.sponge_digest_before_evaluations,
        );

        let npublic_input = vk.index.public;
        let public_inputs = prepared_statement.to_public_input(npublic_input)?;
        let proof_padded = make_padded_proof_from_p2p(proof)?;

        verification_contexts.push((vk.index, proof_padded, public_inputs));
    }

    let proofs: Vec<VerificationContext> = verification_contexts
        .iter()
        .map(|(vk, proof, public_input)| VerificationContext {
            verifier_index: vk,
            proof,
            public_input,
        })
        .collect();

    let result = batch_verify(&proofs);

    Ok(result.is_ok() && checks)
}

/// Dump data when it fails, to reproduce and compare in OCaml
fn dump_zkapp_verification(
    verification_key: &VerificationKey,
    zkapp_statement: &ZkappStatement,
    sideloaded_proof: &PicklesProofProofsVerified2ReprStableV2,
) -> std::io::Result<()> {
    use mina_p2p_messages::binprot;
    use mina_p2p_messages::binprot::macros::{BinProtRead, BinProtWrite};

    #[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
    struct VerifyZkapp {
        vk: v2::MinaBaseVerificationKeyWireStableV1,
        zkapp_statement: v2::MinaBaseZkappStatementStableV2,
        proof: v2::PicklesProofProofsVerified2ReprStableV2,
    }

    let data = VerifyZkapp {
        vk: verification_key.into(),
        zkapp_statement: zkapp_statement.into(),
        proof: sideloaded_proof.clone(),
    };

    let bin = {
        let mut vec = Vec::with_capacity(128 * 1024);
        data.binprot_write(&mut vec)?;
        vec
    };

    let debug_dir = openmina_core::get_debug_dir();
    let filename = debug_dir
        .join(generate_new_filename("verify_zapp", "binprot", &bin)?)
        .to_string_lossy()
        .to_string();
    std::fs::create_dir_all(&debug_dir)?;

    let mut file = std::fs::File::create(filename)?;
    file.write_all(&bin)?;
    file.sync_all()?;

    Ok(())
}

fn generate_new_filename(name: &str, extension: &str, data: &[u8]) -> std::io::Result<String> {
    use crate::proofs::util::sha256_sum;

    let sum = sha256_sum(data);
    for index in 0..100_000 {
        let name = format!("{}_{}_{}.{}", name, sum, index, extension);
        let path = std::path::Path::new(&name);
        if !path.try_exists().unwrap_or(true) {
            return Ok(name);
        }
    }
    Err(std::io::Error::other("no filename available"))
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use std::path::Path;

    use mina_hasher::Fp;
    use mina_p2p_messages::{binprot::BinProtRead, v2};

    use crate::proofs::{provers::devnet_circuit_directory, transaction::tests::panic_in_ci};

    use super::*;

    #[test]
    fn test_verify_zkapp() {
        use mina_p2p_messages::binprot;
        use mina_p2p_messages::binprot::macros::{BinProtRead, BinProtWrite};

        #[derive(Clone, Debug, PartialEq, BinProtRead, BinProtWrite)]
        struct VerifyZkapp {
            vk: v2::MinaBaseVerificationKeyWireStableV1,
            zkapp_statement: v2::MinaBaseZkappStatementStableV2,
            proof: v2::PicklesProofProofsVerified2ReprStableV2,
        }

        let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(devnet_circuit_directory())
            .join("tests");

        let cases = [
            "verify_zapp_4af39d1e141859c964fe32b4e80537d3bd8c32d75e2754c0b869738006d25251_0.binprot",
            "verify_zapp_dc518dc7e0859ea6ffa0cd42637cdcc9c79ab369dfb7ff44c8a89b1219f98728_0.binprot",
            "verify_zapp_9db7255327f342f75d27b5c0f646988ee68c6338f6e26c4dc549675f811b4152_0.binprot",
            "verify_zapp_f2bbc8088654c09314a58c96428f6828d3ee8096b6f34e3a027ad9b028ae22e0_0.binprot",
        ];

        for filename in cases {
            let Ok(file) = std::fs::read(base_dir.join(filename)) else {
                panic_in_ci();
                return;
            };

            let VerifyZkapp {
                vk,
                zkapp_statement,
                proof,
            } = VerifyZkapp::binprot_read(&mut file.as_slice()).unwrap();

            let vk = (&vk).try_into().unwrap();
            let zkapp_statement = (&zkapp_statement).try_into().unwrap();
            let srs = crate::verifier::get_srs::<Fp>();

            let ok = verify_zkapp(&vk, &zkapp_statement, &proof, &srs);
            assert!(ok);
        }
    }

    // #[test]
    // fn test_verification() {
    //     let now = redux::Instant::now();
    //     let verifier_index = get_verifier_index(VerifierKind::Blockchain);
    //     println!("get_verifier_index={:?}", now.elapsed());

    //     let now = redux::Instant::now();
    //     let srs = get_srs::<Fp>();
    //     let srs = srs.lock().unwrap();
    //     println!("get_srs={:?}\n", now.elapsed());

    //     // let now = redux::Instant::now();
    //     // let bytes = verifier_index_to_bytes(&verifier_index);
    //     // println!("verifier_elapsed={:?}", now.elapsed());
    //     // println!("verifier_length={:?}", bytes.len());
    //     // assert_eq!(bytes.len(), 5622520);

    //     // let now = redux::Instant::now();
    //     // let verifier_index = verifier_index_from_bytes(&bytes);
    //     // println!("verifier_deserialize_elapsed={:?}\n", now.elapsed());

    //     // let now = redux::Instant::now();
    //     // let bytes = srs_to_bytes(&srs);
    //     // println!("srs_elapsed={:?}", now.elapsed());
    //     // println!("srs_length={:?}", bytes.len());
    //     // assert_eq!(bytes.len(), 5308513);

    //     // let now = redux::Instant::now();
    //     // let srs: SRS<Vesta> = srs_from_bytes(&bytes);
    //     // println!("deserialize_elapsed={:?}\n", now.elapsed());

    //     // Few blocks headers from berkeleynet
    //     let files = [
    //         include_bytes!("/tmp/block-rampup4.binprot"),
    //         // include_bytes!("../data/5573.binprot"),
    //         // include_bytes!("../data/5574.binprot"),
    //         // include_bytes!("../data/5575.binprot"),
    //         // include_bytes!("../data/5576.binprot"),
    //         // include_bytes!("../data/5577.binprot"),
    //         // include_bytes!("../data/5578.binprot"),
    //         // include_bytes!("../data/5579.binprot"),
    //         // include_bytes!("../data/5580.binprot"),
    //     ];

    //     use mina_p2p_messages::binprot::BinProtRead;
    //     use crate::proofs::accumulator_check::accumulator_check;

    //     for file in files {
    //         let header = MinaBlockHeaderStableV2::binprot_read(&mut file.as_slice()).unwrap();

    //         let now = redux::Instant::now();
    //         let accum_check = accumulator_check(&*srs, &header.protocol_state_proof.0);
    //         println!("accumulator_check={:?}", now.elapsed());

    //         let now = redux::Instant::now();
    //         let verified = super::verify_block(&header, &verifier_index, &*srs);

    //         // let verified = crate::verify(&header, &verifier_index);
    //         println!("snark::verify={:?}", now.elapsed());

    //         assert!(accum_check);
    //         assert!(verified);
    //     }
    // }

    // #[test]
    // fn test_verifier_index_deterministic() {
    //     let mut nruns = 0;
    //     let nruns = &mut nruns;

    //     let mut hash_verifier_index = || {
    //         *nruns += 1;
    //         let verifier_index = get_verifier_index();
    //         let bytes = verifier_index_to_bytes(&verifier_index);

    //         let mut hasher = DefaultHasher::new();
    //         bytes.hash(&mut hasher);
    //         hasher.finish()
    //     };

    //     assert_eq!(hash_verifier_index(), hash_verifier_index());
    //     assert_eq!(*nruns, 2);
    // }
}

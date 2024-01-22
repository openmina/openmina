use std::{array, rc::Rc};

use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

use crate::{
    proofs::{
        accumulator_check,
        step::{expand_deferred, StatementProofState},
        transaction::endos,
        unfinalized::AllEvals,
        verifier_index::make_zkapp_verifier_index,
        wrap::Domain,
        BACKEND_TICK_ROUNDS_N,
    },
    scan_state::{
        protocol_state::MinaHash,
        scan_state::transaction_snark::{SokDigest, Statement},
        transaction_logic::zkapp_statement::ZkappStatement,
    },
    VerificationKey,
};

use super::{
    field::FieldWitness,
    public_input::plonk_checks::make_shifts,
    step::{step_verifier::PlonkDomain, ExpandDeferredParams},
    to_field_elements::ToFieldElements,
    transaction::{InnerCurve, PlonkVerificationKeyEvals},
    util::{extract_bulletproof, extract_polynomial_commitment, u64_to_field},
    VerifierSRS,
};
use kimchi::{
    circuits::{polynomials::permutation::eval_zk_polynomial, wires::PERMUTS},
    error::VerifyError,
    mina_curves::pasta::Pallas,
    proof::{PointEvaluations, ProofEvaluations},
};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    v2::{
        CompositionTypesDigestConstantStableV1, MinaBlockHeaderStableV2,
        PicklesProofProofsVerified2ReprStableV2,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags,
        TransactionSnarkProofStableV2,
    },
};

use super::{prover::make_padded_proof_from_p2p, ProverProof, VerifierIndex};

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
                0 | 1 | 2 => lookups_per_row_2,
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
    .all(|b| *b == true)
}

pub fn prev_evals_from_p2p<F: FieldWitness>(
    evals: &PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
) -> ProofEvaluations<PointEvaluations<Vec<F>>> {
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

    let of = |(zeta, zeta_omega): &(Vec<BigInt>, Vec<BigInt>)| -> PointEvaluations<Vec<F>> {
        PointEvaluations {
            zeta: zeta.iter().map(BigInt::to_field).collect(),
            zeta_omega: zeta_omega.iter().map(BigInt::to_field).collect(),
        }
    };

    let of_opt = |v: &Option<(Vec<BigInt>, Vec<BigInt>)>| v.as_ref().map(of);

    ProofEvaluations {
        w: array::from_fn(|i| of(&w[i])),
        z: of(z),
        s: array::from_fn(|i| of(&s[i])),
        coefficients: array::from_fn(|i| of(&coefficients[i])),
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
        lookup_sorted: array::from_fn(|i| of_opt(&lookup_sorted[i])),
        runtime_lookup_table: of_opt(runtime_lookup_table),
        runtime_lookup_table_selector: of_opt(runtime_lookup_table_selector),
        xor_lookup_selector: of_opt(xor_lookup_selector),
        lookup_gate_lookup_selector: of_opt(lookup_gate_lookup_selector),
        range_check_lookup_selector: of_opt(range_check_lookup_selector),
        foreign_field_mul_lookup_selector: of_opt(foreign_field_mul_lookup_selector),
    }
}

pub fn prev_evals_to_p2p(
    evals: &ProofEvaluations<[Fp; 2]>,
) -> PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
    let ProofEvaluations {
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

    let of = |[zeta, zeta_omega]: &[Fp; 2]| -> (Vec<BigInt>, Vec<BigInt>) {
        (vec![zeta.into()], vec![zeta_omega.into()])
    };

    let of_opt = |v: &Option<[Fp; 2]>| v.as_ref().map(of);

    PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
        w: PaddedSeq(array::from_fn(|i| of(&w[i]))),
        z: of(z),
        s: PaddedSeq(array::from_fn(|i| of(&s[i]))),
        coefficients: PaddedSeq(array::from_fn(|i| of(&coefficients[i]))),
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
        lookup_sorted: PaddedSeq(array::from_fn(|i| of_opt(&lookup_sorted[i]))),
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
        todo!()
    }
    fn generator(&self) -> F {
        todo!()
    }
    fn shifts(&self) -> &[F; PERMUTS] {
        self.shifts.shifts()
    }
    fn log2_size(&self) -> u64 {
        todo!()
    }
}

// TODO: `domain_log2` and `srs_length_log2` might be the same here ? Remove one or the other
pub fn make_scalars_env<F: FieldWitness, const NLIMB: usize>(
    minimal: &PlonkMinimal<F, NLIMB>,
    domain_log2: u8,
    srs_length_log2: u64,
) -> ScalarsEnv<F> {
    let domain: Radix2EvaluationDomain<F> =
        Radix2EvaluationDomain::new(1 << domain_log2 as u64).unwrap();

    let zk_polynomial = eval_zk_polynomial(domain, minimal.zeta);
    let zeta_to_n_minus_1 = domain.evaluate_vanishing_polynomial(minimal.zeta);

    let (_w4, w3, _w2, _w1) = {
        let gen = domain.group_gen;
        let w1 = F::one() / gen;
        let w2 = w1.square();
        let w3 = w2 * w1;
        // let w4 = (); // unused for now
        // let w4 = w3 * w1;

        ((), w3, w2, w1)
    };

    let shifts = make_shifts(&domain);
    let domain = Rc::new(LimitedDomain { domain, shifts });

    ScalarsEnv {
        zk_polynomial,
        zeta_to_n_minus_1,
        srs_length_log2,
        domain,
        omega_to_minus_3: w3,
        feature_flags: None,
        unnormalized_lagrange_basis: None,
        vanishes_on_last_4_rows: F::one(),
    }
}

fn get_message_for_next_step_proof<'a, AppState>(
    messages_for_next_step_proof: &PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
    commitments: &'a PlonkVerificationKeyEvals<Fp>,
    app_state: &'a AppState,
) -> MessagesForNextStepProof<'a, AppState>
where
    AppState: ToFieldElements<Fp>,
{
    let PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
        app_state: _, // unused
        challenge_polynomial_commitments,
        old_bulletproof_challenges,
    } = messages_for_next_step_proof;

    let (_, endo) = endos::<Fq>();

    let challenge_polynomial_commitments: Vec<InnerCurve<Fp>> =
        extract_polynomial_commitment(challenge_polynomial_commitments);
    let old_bulletproof_challenges: Vec<[Fp; 16]> =
        extract_bulletproof(old_bulletproof_challenges, &endo);
    let dlog_plonk_index = commitments;

    MessagesForNextStepProof {
        app_state,
        dlog_plonk_index,
        challenge_polynomial_commitments,
        old_bulletproof_challenges,
    }
}

fn get_message_for_next_wrap_proof(
    PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
        challenge_polynomial_commitment,
        old_bulletproof_challenges,
    }: &PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
) -> MessagesForNextWrapProof {
    let challenge_polynomial_commitments: Vec<InnerCurve<Fq>> =
        extract_polynomial_commitment(&[challenge_polynomial_commitment.clone()]);

    let (_, endo) = endos::<Fp>();

    let old_bulletproof_challenges: Vec<[Fq; 15]> = extract_bulletproof(
        &[
            old_bulletproof_challenges[0].0.clone(),
            old_bulletproof_challenges[1].0.clone(),
        ],
        &endo,
    );

    MessagesForNextWrapProof {
        challenge_polynomial_commitment: challenge_polynomial_commitments[0].clone(),
        old_bulletproof_challenges,
    }
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
    let sponge_digest_before_evaluations: [u64; 4] = array::from_fn(|i| digest[i].as_u64());

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
    verifier_index: &VerifierIndex,
    proof: &ProverProof,
    public_input: &[Fq],
) -> Result<(), VerifyError> {
    use kimchi::groupmap::GroupMap;
    use kimchi::mina_curves::pasta::PallasParameters;
    use mina_poseidon::sponge::{DefaultFqSponge, DefaultFrSponge};

    type SpongeParams = mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    type EFqSponge = DefaultFqSponge<PallasParameters, SpongeParams>;
    type EFrSponge = DefaultFrSponge<Fq, SpongeParams>;

    let group_map = GroupMap::<Fp>::setup();

    kimchi::verifier::verify::<Pallas, EFqSponge, EFrSponge>(
        &group_map,
        verifier_index,
        proof,
        public_input,
    )
}

fn run_checks(
    proof: &PicklesProofProofsVerified2ReprStableV2,
    verifier_index: &VerifierIndex,
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

fn compute_deferred_values(proof: &PicklesProofProofsVerified2ReprStableV2) -> DeferredValues<Fp> {
    let bulletproof_challenges: Vec<Fp> = proof
        .statement
        .proof_state
        .deferred_values
        .bulletproof_challenges
        .iter()
        .map(|chal| {
            let prechallenge = &chal.prechallenge.inner;
            let prechallenge: [u64; 2] = array::from_fn(|k| prechallenge[k].as_u64());
            u64_to_field(&prechallenge)
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
                    .map(|v| u64_to_field(&v.prechallenge.inner.0.map(|v| v.as_u64())))
            })
            .collect();
        let proof_state: StatementProofState = (&proof.statement.proof_state).into();
        let evals: AllEvals<Fp> = (&proof.prev_evals).into();

        expand_deferred(ExpandDeferredParams {
            evals: &evals,
            old_bulletproof_challenges: &old_bulletproof_challenges,
            proof_state: &proof_state,
        })
    };

    DeferredValues {
        bulletproof_challenges,
        ..deferred_values
    }
}

/// https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/pickles/verification_key.mli#L30
pub struct VK<'a> {
    pub commitments: PlonkVerificationKeyEvals<Fp>,
    pub index: &'a VerifierIndex,
    pub data: (), // Unused in proof verification
}

pub fn verify_block(
    header: &MinaBlockHeaderStableV2,
    verifier_index: &VerifierIndex,
    srs: &VerifierSRS,
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

    let accum_check = accumulator_check::accumulator_check(srs, protocol_state_proof);
    let verified = verify_impl(&MinaHash::hash(protocol_state), protocol_state_proof, &vk);
    accum_check && verified
}

pub fn verify_transaction<'a>(
    proofs: impl IntoIterator<Item = (&'a Statement<SokDigest>, &'a TransactionSnarkProofStableV2)>,
    verifier_index: &VerifierIndex,
    srs: &VerifierSRS,
) -> bool {
    let vk = VK {
        commitments: PlonkVerificationKeyEvals::from(verifier_index),
        index: verifier_index,
        data: (),
    };

    proofs.into_iter().all(|(statement, transaction_proof)| {
        let accum_check = accumulator_check::accumulator_check(srs, transaction_proof);
        let verified = verify_impl(statement, transaction_proof, &vk);
        accum_check && verified
    })
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/crypto/kimchi_bindings/stubs/src/pasta_fq_plonk_proof.rs#L116
pub fn verify_zkapp(
    verification_key: &VerificationKey,
    zkapp_statement: ZkappStatement,
    sideloaded_proof: &PicklesProofProofsVerified2ReprStableV2,
    srs: &VerifierSRS,
) -> bool {
    let verifier_index = make_zkapp_verifier_index(verification_key);
    // https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/pickles/pickles.ml#LL260C1-L274C18
    let vk = VK {
        commitments: verification_key.wrap_index.clone(),
        index: &verifier_index,
        data: (),
    };

    let accum_check = accumulator_check::accumulator_check(srs, sideloaded_proof);
    let verified = verify_impl(&zkapp_statement, sideloaded_proof, &vk);

    let ok = accum_check && verified;

    eprintln!("verify_zkapp OK={:?}", ok);

    ok
}

fn verify_impl<AppState>(
    app_state: &AppState,
    proof: &PicklesProofProofsVerified2ReprStableV2,
    vk: &VK,
) -> bool
where
    AppState: ToFieldElements<Fp>,
{
    let deferred_values = compute_deferred_values(proof);
    let checks = run_checks(proof, vk.index);

    let message_for_next_step_proof = get_message_for_next_step_proof(
        &proof.statement.messages_for_next_step_proof,
        &vk.commitments,
        app_state,
    );

    let message_for_next_wrap_proof =
        get_message_for_next_wrap_proof(&proof.statement.proof_state.messages_for_next_wrap_proof);

    let prepared_statement = get_prepared_statement(
        &message_for_next_step_proof,
        &message_for_next_wrap_proof,
        deferred_values,
        &proof.statement.proof_state.sponge_digest_before_evaluations,
    );

    let npublic_input = vk.index.public;
    let public_inputs = prepared_statement.to_public_input(npublic_input);
    let proof = make_padded_proof_from_p2p(proof);

    let result = verify_with(vk.index, &proof, &public_inputs);

    if let Err(e) = result {
        eprintln!("verify error={:?}", e);
    };

    result.is_ok() && checks
}

// #[cfg(test)]
// mod tests {
//     use std::{
//         collections::hash_map::DefaultHasher,
//         hash::{Hash, Hasher},
//     };

//     // use binprot::BinProtRead;
//     use mina_curves::pasta::{Vesta, Fq};
//     use mina_hasher::Fp;
//     use mina_p2p_messages::v2::MinaBlockHeaderStableV2;
//     use poly_commitment::srs::SRS;

//     use crate::{
//         proofs::{caching::{
//             srs_from_bytes, srs_to_bytes, verifier_index_from_bytes, verifier_index_to_bytes,
//         }, verifier_index::{get_verifier_index, VerifierKind}}, verifier::get_srs,
//         // get_srs, get_verifier_index,
//     };

//     #[cfg(target_family = "wasm")]
//     use wasm_bindgen_test::wasm_bindgen_test as test;

//     #[test]
//     fn test_verification() {
//         let now = std::time::Instant::now();
//         let verifier_index = get_verifier_index(VerifierKind::Blockchain);
//         println!("get_verifier_index={:?}", now.elapsed());

//         let now = std::time::Instant::now();
//         let srs = get_srs::<Fp>();
//         let srs = srs.lock().unwrap();
//         println!("get_srs={:?}\n", now.elapsed());

//         // let now = std::time::Instant::now();
//         // let bytes = verifier_index_to_bytes(&verifier_index);
//         // println!("verifier_elapsed={:?}", now.elapsed());
//         // println!("verifier_length={:?}", bytes.len());
//         // assert_eq!(bytes.len(), 5622520);

//         // let now = std::time::Instant::now();
//         // let verifier_index = verifier_index_from_bytes(&bytes);
//         // println!("verifier_deserialize_elapsed={:?}\n", now.elapsed());

//         // let now = std::time::Instant::now();
//         // let bytes = srs_to_bytes(&srs);
//         // println!("srs_elapsed={:?}", now.elapsed());
//         // println!("srs_length={:?}", bytes.len());
//         // assert_eq!(bytes.len(), 5308513);

//         // let now = std::time::Instant::now();
//         // let srs: SRS<Vesta> = srs_from_bytes(&bytes);
//         // println!("deserialize_elapsed={:?}\n", now.elapsed());

//         // Few blocks headers from berkeleynet
//         let files = [
//             include_bytes!("/tmp/block-rampup4.binprot"),
//             // include_bytes!("../data/5573.binprot"),
//             // include_bytes!("../data/5574.binprot"),
//             // include_bytes!("../data/5575.binprot"),
//             // include_bytes!("../data/5576.binprot"),
//             // include_bytes!("../data/5577.binprot"),
//             // include_bytes!("../data/5578.binprot"),
//             // include_bytes!("../data/5579.binprot"),
//             // include_bytes!("../data/5580.binprot"),
//         ];

//         use mina_p2p_messages::binprot::BinProtRead;
//         use crate::proofs::accumulator_check::accumulator_check;

//         for file in files {
//             let header = MinaBlockHeaderStableV2::binprot_read(&mut file.as_slice()).unwrap();

//             let now = std::time::Instant::now();
//             let accum_check = accumulator_check(&*srs, &header.protocol_state_proof.0);
//             println!("accumulator_check={:?}", now.elapsed());

//             let now = std::time::Instant::now();
//             let verified = super::verify_block(&header, &verifier_index, &*srs);

//             // let verified = crate::verify(&header, &verifier_index);
//             println!("snark::verify={:?}", now.elapsed());

//             assert!(accum_check);
//             assert!(verified);
//         }
//     }

//     #[test]
//     fn test_verifier_index_deterministic() {
//         let mut nruns = 0;
//         let nruns = &mut nruns;

//         let mut hash_verifier_index = || {
//             *nruns += 1;
//             let verifier_index = get_verifier_index();
//             let bytes = verifier_index_to_bytes(&verifier_index);

//             let mut hasher = DefaultHasher::new();
//             bytes.hash(&mut hasher);
//             hasher.finish()
//         };

//         assert_eq!(hash_verifier_index(), hash_verifier_index());
//         assert_eq!(*nruns, 2);
//     }
// }

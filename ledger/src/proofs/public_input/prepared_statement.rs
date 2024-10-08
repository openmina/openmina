use ark_ff::{BigInteger256, Zero};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    CompositionTypesBranchDataStableV1, PicklesBaseProofsVerifiedStableV1,
};

use crate::proofs::{
    field::{CircuitVar, FieldWitness},
    step::{FeatureFlags, OptFlag, Packed},
    to_field_elements::ToFieldElements,
    util::u64_to_field,
};

#[derive(Clone, Debug)]
pub struct Plonk<F: FieldWitness> {
    pub alpha: [u64; 2],
    pub beta: [u64; 2],
    pub gamma: [u64; 2],
    pub zeta: [u64; 2],
    pub zeta_to_srs_length: F::Shifting,
    pub zeta_to_domain_size: F::Shifting,
    pub perm: F::Shifting,
    pub lookup: Option<[u64; 2]>,
    pub feature_flags: FeatureFlags<bool>,
}

#[derive(Clone, Debug)]
pub struct DeferredValues<F: FieldWitness> {
    pub plonk: Plonk<F>,
    pub combined_inner_product: F::Shifting,
    pub b: F::Shifting,
    pub xi: [u64; 2],
    pub bulletproof_challenges: Vec<F>,
    pub branch_data: CompositionTypesBranchDataStableV1,
}

// REVIEW(dw): ProofState is the state of the previous proof.
// REVIEW(dw): it is always Fp (i.e. scalar field of Vesta/Step)?
// Should it not be parametrized by the field for the Wrap?
#[derive(Clone, Debug)]
pub struct ProofState {
    pub deferred_values: DeferredValues<Fp>,
    pub sponge_digest_before_evaluations: [u64; 4],
    pub messages_for_next_wrap_proof: [u64; 4],
}

#[derive(Debug)]
pub struct PreparedStatement {
    pub proof_state: ProofState,
    pub messages_for_next_step_proof: [u64; 4],
}

impl PreparedStatement {
    /// Implementation of `tock_unpadded_public_input_of_statement`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/common.ml#L202
    pub fn to_public_input(&self, npublic_input: usize) -> Vec<Fq> {
        let PreparedStatement {
            proof_state:
                ProofState {
                    deferred_values:
                        DeferredValues {
                            plonk:
                                Plonk {
                                    alpha,
                                    beta,
                                    gamma,
                                    zeta,
                                    zeta_to_srs_length,
                                    zeta_to_domain_size,
                                    perm,
                                    lookup,
                                    feature_flags,
                                },
                            combined_inner_product,
                            b,
                            xi,
                            bulletproof_challenges,
                            branch_data:
                                CompositionTypesBranchDataStableV1 {
                                    proofs_verified,
                                    domain_log2,
                                },
                        },
                    sponge_digest_before_evaluations,
                    messages_for_next_wrap_proof,
                },
            messages_for_next_step_proof,
        } = &self;

        // We sort the fields in the same order as here:
        // https://github.com/MinaProtocol/mina/blob/c824be7d80db1d290e0d48cbc920182d07de0330/src/lib/pickles/composition_types/composition_types.ml#L739

        let mut fields: Vec<Fq> = Vec::with_capacity(npublic_input);

        let to_fq = |fp: Fp| -> Fq {
            let bigint: BigInteger256 = fp.into();
            bigint.into()
        };

        // Fp
        {
            fields.push(to_fq(combined_inner_product.shifted));
            fields.push(to_fq(b.shifted));
            fields.push(to_fq(zeta_to_srs_length.shifted));
            fields.push(to_fq(zeta_to_domain_size.shifted));
            fields.push(to_fq(perm.shifted));
        }

        // Challenge
        {
            fields.push(u64_to_field(beta));
            fields.push(u64_to_field(gamma));
        }

        // Scalar challenge
        {
            fields.push(u64_to_field(alpha));
            fields.push(u64_to_field(zeta));
            fields.push(u64_to_field(xi));
        }

        // Digest
        {
            fields.push(u64_to_field(sponge_digest_before_evaluations));
            fields.push(u64_to_field(messages_for_next_wrap_proof));
            fields.push(u64_to_field(messages_for_next_step_proof));
        }

        fields.extend(bulletproof_challenges.iter().copied().map(to_fq));

        // Index
        {
            // https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles_base/proofs_verified.ml#L58
            let proofs_verified = match proofs_verified {
                PicklesBaseProofsVerifiedStableV1::N0 => 0b00,
                PicklesBaseProofsVerifiedStableV1::N1 => 0b10,
                PicklesBaseProofsVerifiedStableV1::N2 => 0b11,
            };
            // https://github.com/MinaProtocol/mina/blob/c824be7d80db1d290e0d48cbc920182d07de0330/src/lib/pickles/composition_types/branch_data.ml#L63
            let domain_log2: u8 = domain_log2.0.into();
            let domain_log2: u64 = domain_log2 as u64;
            let branch_data: u64 = (domain_log2 << 2) | proofs_verified;

            fields.push(u64_to_field(&[branch_data]));
        }

        let lookup_value = lookup.as_ref().map(u64_to_field);

        feature_flags.to_field_elements(&mut fields);

        let FeatureFlags {
            range_check0,
            range_check1,
            foreign_field_add: _,
            foreign_field_mul,
            xor,
            rot,
            lookup,
            runtime_tables: _,
        } = feature_flags;

        // https://github.com/MinaProtocol/mina/blob/dc6bf78b8ddbbca3a1a248971b76af1514bf05aa/src/lib/pickles/composition_types/composition_types.ml#L146
        let uses_lookup = [
            range_check0,
            range_check1,
            foreign_field_mul,
            xor,
            rot,
            lookup,
        ]
        .iter()
        .any(|b| **b);

        uses_lookup.to_field_elements(&mut fields);

        if uses_lookup {
            fields.push(lookup_value.unwrap());
        } else {
            fields.push(Fq::zero());
        }

        assert_eq!(fields.len(), npublic_input);

        fields
    }

    pub fn to_public_input_cvar(
        &self,
        hack_feature_flags: OptFlag,
        npublic_input: usize,
    ) -> Vec<Packed> {
        let PreparedStatement {
            proof_state:
                ProofState {
                    deferred_values:
                        DeferredValues {
                            plonk:
                                Plonk {
                                    alpha,
                                    beta,
                                    gamma,
                                    zeta,
                                    zeta_to_srs_length,
                                    zeta_to_domain_size,
                                    perm,
                                    lookup: _,
                                    feature_flags: _,
                                },
                            combined_inner_product,
                            b,
                            xi,
                            bulletproof_challenges,
                            branch_data:
                                CompositionTypesBranchDataStableV1 {
                                    proofs_verified,
                                    domain_log2,
                                },
                        },
                    sponge_digest_before_evaluations,
                    messages_for_next_wrap_proof,
                },
            messages_for_next_step_proof,
        } = &self;

        // We sort the fields in the same order as here:
        // https://github.com/MinaProtocol/mina/blob/c824be7d80db1d290e0d48cbc920182d07de0330/src/lib/pickles/composition_types/composition_types.ml#L739

        let mut fields: Vec<Packed> = Vec::with_capacity(npublic_input);

        let to_fq = |fp: Fp| -> Fq {
            let bigint: BigInteger256 = fp.into();
            bigint.into()
        };

        let var = |x| Packed::Field(CircuitVar::Var(x));
        let bits = |n, x| Packed::PackedBits(CircuitVar::Var(x), n);

        // Fp
        {
            fields.push(var(to_fq(combined_inner_product.shifted)));
            fields.push(var(to_fq(b.shifted)));
            fields.push(var(to_fq(zeta_to_srs_length.shifted)));
            fields.push(var(to_fq(zeta_to_domain_size.shifted)));
            fields.push(var(to_fq(perm.shifted)));
        }

        // Challenge
        {
            fields.push(bits(128, u64_to_field(beta)));
            fields.push(bits(128, u64_to_field(gamma)));
        }

        // Scalar challenge
        {
            fields.push(bits(128, u64_to_field(alpha)));
            fields.push(bits(128, u64_to_field(zeta)));
            fields.push(bits(128, u64_to_field(xi)));
        }

        // Digest
        {
            fields.push(bits(255, u64_to_field(sponge_digest_before_evaluations)));
            fields.push(bits(255, u64_to_field(messages_for_next_wrap_proof)));
            fields.push(bits(255, u64_to_field(messages_for_next_step_proof)));
        }

        fields.extend(
            bulletproof_challenges
                .iter()
                .copied()
                .map(|v| bits(128, to_fq(v))),
        );

        // Index
        {
            // https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles_base/proofs_verified.ml#L58
            let proofs_verified = match proofs_verified {
                PicklesBaseProofsVerifiedStableV1::N0 => 0b00,
                PicklesBaseProofsVerifiedStableV1::N1 => 0b10,
                PicklesBaseProofsVerifiedStableV1::N2 => 0b11,
            };
            // https://github.com/MinaProtocol/mina/blob/c824be7d80db1d290e0d48cbc920182d07de0330/src/lib/pickles/composition_types/branch_data.ml#L63
            let domain_log2: u8 = domain_log2.0.into();
            let domain_log2: u64 = domain_log2 as u64;
            let branch_data: u64 = (domain_log2 << 2) | proofs_verified;

            fields.push(bits(10, u64_to_field(&[branch_data])));
        }

        // TODO: Find out how this padding works, it's probably related to features/lookup
        let zero = Fq::zero();
        let circuit_var = match hack_feature_flags {
            OptFlag::Yes => todo!(),
            OptFlag::No => CircuitVar::Constant,
            OptFlag::Maybe => CircuitVar::Var,
        };
        while fields.len() < npublic_input - 1 {
            fields.push(Packed::PackedBits(circuit_var(zero), 1));
        }
        fields.push(Packed::PackedBits(circuit_var(zero), 128));

        fields
    }
}

use ark_ff::BigInteger256;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    CompositionTypesBranchDataStableV1, PicklesBaseProofsVerifiedStableV1,
};

use crate::u64_to_field;

use super::plonk_checks::ShiftedValue;

pub struct Plonk {
    pub alpha: [u64; 2],
    pub beta: [u64; 2],
    pub gamma: [u64; 2],
    pub zeta: [u64; 2],
    pub zeta_to_srs_length: ShiftedValue<Fp>,
    pub zeta_to_domain_size: ShiftedValue<Fp>,
    pub poseidon_selector: ShiftedValue<Fp>,
    pub vbmul: ShiftedValue<Fp>,
    pub complete_add: ShiftedValue<Fp>,
    pub endomul: ShiftedValue<Fp>,
    pub endomul_scalar: ShiftedValue<Fp>,
    pub perm: ShiftedValue<Fp>,
    pub generic: [ShiftedValue<Fp>; 9],
    pub lookup: (),
}

pub struct DeferredValues {
    pub plonk: Plonk,
    pub combined_inner_product: ShiftedValue<Fp>,
    pub b: ShiftedValue<Fp>,
    pub xi: [u64; 2],
    pub bulletproof_challenges: Vec<[u64; 2]>,
    pub branch_data: CompositionTypesBranchDataStableV1,
}

pub struct ProofState {
    pub deferred_values: DeferredValues,
    pub sponge_digest_before_evaluations: [u64; 4],
    pub messages_for_next_wrap_proof: [u64; 4],
}

pub struct PreparedStatement {
    pub proof_state: ProofState,
    pub messages_for_next_step_proof: [u64; 4],
}

impl PreparedStatement {
    /// Implementation of `tock_unpadded_public_input_of_statement`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/common.ml#L202
    pub fn to_public_input(&self) -> Vec<Fq> {
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
                                    poseidon_selector,
                                    vbmul,
                                    complete_add,
                                    endomul,
                                    endomul_scalar,
                                    perm,
                                    generic,
                                    lookup: _, // `lookup` is of type `()`
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

        let mut fields: Vec<Fq> = Vec::with_capacity(47);

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
            fields.push(to_fq(poseidon_selector.shifted));
            fields.push(to_fq(vbmul.shifted));
            fields.push(to_fq(complete_add.shifted));
            fields.push(to_fq(endomul.shifted));
            fields.push(to_fq(endomul_scalar.shifted));
            fields.push(to_fq(perm.shifted));
            fields.extend(generic.iter().map(|g| to_fq(g.shifted)));
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

        fields.extend(bulletproof_challenges.iter().map(u64_to_field::<Fq, 2>));

        // Index
        {
            // https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles_base/proofs_verified.ml#L58
            let proofs_verified = match proofs_verified.0 {
                PicklesBaseProofsVerifiedStableV1::N0 => 0b00,
                PicklesBaseProofsVerifiedStableV1::N1 => 0b01, // Bits are reversed
                PicklesBaseProofsVerifiedStableV1::N2 => 0b11,
            };
            // https://github.com/MinaProtocol/mina/blob/c824be7d80db1d290e0d48cbc920182d07de0330/src/lib/pickles/composition_types/branch_data.ml#L63
            let domain_log2: u8 = domain_log2.0.into();
            let domain_log2: u64 = domain_log2 as u64;
            let branch_data: u64 = (domain_log2 << 2) | proofs_verified;

            fields.push(u64_to_field(&[branch_data]));
        }

        // TODO: Not sure how that padding works, check further
        fields.push(0.into());
        fields.push(0.into());
        fields.push(0.into());

        assert_eq!(fields.len(), 47);

        fields
    }
}

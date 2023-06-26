use crate::{
    hash_input::{Inputs, ToInput},
    v2::{
        MinaBaseVerificationKeyWireStableV1, MinaBaseVerificationKeyWireStableV1WrapIndex,
        PicklesBaseProofsVerifiedStableV1,
    },
};

impl ToInput for MinaBaseVerificationKeyWireStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBaseVerificationKeyWireStableV1 {
            max_proofs_verified,
            actual_wrap_domain_size,
            wrap_index,
        } = self;
        to_input_fields!(
            inputs,
            max_proofs_verified,
            actual_wrap_domain_size,
            wrap_index
        );
    }
}

impl ToInput for PicklesBaseProofsVerifiedStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let bits = match self {
            PicklesBaseProofsVerifiedStableV1::N0 => [true, false, false],
            PicklesBaseProofsVerifiedStableV1::N1 => [false, true, false],
            PicklesBaseProofsVerifiedStableV1::N2 => [false, false, true],
        };
        for bit in bits {
            inputs.append_bool(bit);
        }
    }
}

impl ToInput for MinaBaseVerificationKeyWireStableV1WrapIndex {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBaseVerificationKeyWireStableV1WrapIndex {
            sigma_comm,
            coefficients_comm,
            generic_comm,
            psm_comm,
            complete_add_comm,
            mul_comm,
            emul_comm,
            endomul_scalar_comm,
        } = self;
        to_input_fields!(
            inputs,
            sigma_comm,
            coefficients_comm,
            generic_comm,
            psm_comm,
            complete_add_comm,
            mul_comm,
            emul_comm,
            endomul_scalar_comm
        );
    }
}

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

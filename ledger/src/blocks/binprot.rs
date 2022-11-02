use mina_hasher::Fp;
use serde::{Deserialize, Serialize};

use crate::{
    BigInt, CurveAffine, MessagesForNextStepProof, MinaBaseVerificationKeyWireStableV1WrapIndex,
    PlonkVerificationKeyEvals,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
    pub app_state: BigInt,
    pub dlog_plonk_index: MinaBaseVerificationKeyWireStableV1WrapIndex,
    pub challenge_polynomial_commitments: Vec<(BigInt, BigInt)>,
    pub old_bulletproof_challenges: Vec<Vec<BigInt>>,
}

impl From<PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof>
    for MessagesForNextStepProof
{
    fn from(value: PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof) -> Self {
        Self {
            app_state: [value.app_state.into(); 1],
            dlog_plonk_index: {
                // TODO: Refactor with Account

                let idx = value.dlog_plonk_index;

                #[rustfmt::skip]
                let sigma = [
                    idx.sigma_comm.0.into(),
                    idx.sigma_comm.1.0.into(),
                    idx.sigma_comm.1.1.0.into(),
                    idx.sigma_comm.1.1.1.0.into(),
                    idx.sigma_comm.1.1.1.1.0.into(),
                    idx.sigma_comm.1.1.1.1.1.0.into(),
                    idx.sigma_comm.1.1.1.1.1.1.0.into(),
                ];

                #[rustfmt::skip]
                let coefficients = [
                    idx.coefficients_comm.0.into(),
                    idx.coefficients_comm.1.0.into(),
                    idx.coefficients_comm.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                ];

                PlonkVerificationKeyEvals {
                    sigma,
                    coefficients,
                    generic: idx.generic_comm.into(),
                    psm: idx.psm_comm.into(),
                    complete_add: idx.complete_add_comm.into(),
                    mul: idx.mul_comm.into(),
                    emul: idx.emul_comm.into(),
                    endomul_scalar: idx.endomul_scalar_comm.into(),
                }
            },
            challenge_polynomial_commitments: {
                let vec = value
                    .challenge_polynomial_commitments
                    .into_iter()
                    .map(|a| a.into())
                    .collect::<Vec<CurveAffine<Fp>>>();

                vec.try_into().expect("wrong vec length")
            },
            old_bulletproof_challenges: {
                value
                    .old_bulletproof_challenges
                    .into_iter()
                    .map(|a| {
                        let a: [_; 16] = a
                            .into_iter()
                            .map(|f| f.into())
                            .collect::<Vec<Fp>>()
                            .try_into()
                            .expect("wrong length");
                        a
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("wrong vec length")
            },
        }
    }
}

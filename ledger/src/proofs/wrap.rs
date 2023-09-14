use kimchi::proof::ProofEvaluations;
use mina_hasher::Fp;
use mina_p2p_messages::v2::PicklesProofProofsVerified2ReprStableV2;

// use crate::proofs::{
//     public_input::plonk_checks::ft_eval0,
//     util::{challenge_polynomial, proof_evaluation_to_list},
// };

use super::public_input::plonk_checks::{PlonkMinimal, ScalarsEnv};

pub struct CombinedInnerProductParams<'a> {
    pub env: &'a ScalarsEnv,
    pub evals: &'a ProofEvaluations<[Fp; 2]>,
    pub minimal: &'a PlonkMinimal,
    pub proof: &'a PicklesProofProofsVerified2ReprStableV2,
    pub r: Fp,
    pub old_bulletproof_challenges: &'a [[Fp; 16]],
    // pub xi: Fp,
    pub zetaw: Fp,
}

// /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/wrap.ml#L37
// pub fn combined_inner_product(params: CombinedInnerProductParams) -> Fp {
//     let CombinedInnerProductParams {
//         env,
//         old_bulletproof_challenges,
//         evals,
//         minimal,
//         proof,
//         r,
//         // xi,
//         zetaw,
//     } = params;

//     let ft_eval0 = ft_eval0(
//         env,
//         evals,
//         minimal,
//         proof.prev_evals.evals.public_input.0.to_field(),
//     );

//     let challenge_polys: Vec<_> = old_bulletproof_challenges
//         .iter()
//         .map(|v| challenge_polynomial(v))
//         .collect();

//     let a = proof_evaluation_to_list(evals);
//     let ft_eval1: Fp = proof.prev_evals.ft_eval1.to_field();

//     enum WhichEval {
//         First,
//         Second,
//     }

//     let combine = |which_eval: WhichEval, ft: Fp, pt: Fp| {
//         let f = |[x, y]: &[Fp; 2]| match which_eval {
//             WhichEval::First => *x,
//             WhichEval::Second => *y,
//         };
//         let a: Vec<_> = a.iter().map(f).collect();
//         let public_input = &proof.prev_evals.evals.public_input;
//         let public_input: [Fp; 2] = [public_input.0.to_field(), public_input.1.to_field()];

//         let mut v: Vec<_> = challenge_polys
//             .iter()
//             .map(|f| f(pt))
//             .chain([f(&public_input), ft])
//             .chain(a)
//             .collect();

//         v.reverse();
//         let (init, rest) = v.split_at(1);
//         rest.iter().fold(init[0], |acc, fx| *fx + (xi * acc))
//     };

//     combine(WhichEval::First, ft_eval0, minimal.zeta)
//         + (r * combine(WhichEval::Second, ft_eval1, zetaw))
// }

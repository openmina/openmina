use crate::ArithmeticSpongeParams;

use super::{
    witness::{field, Boolean, FieldWitness, Witness},
    wrap::CircuitVar,
};

const M: usize = 3;
const CAPACITY: usize = 1;
const RATE: usize = M - CAPACITY;
const PERM_ROUNDS_FULL: usize = 55;

pub enum SpongeState<F: FieldWitness> {
    Absorbing {
        next_index: Boolean,
        xs: Vec<(CircuitVar<Boolean>, F)>,
    },
    Squeezed(usize),
}

pub struct OptSponge<F: FieldWitness> {
    pub state: [F; M],
    params: &'static ArithmeticSpongeParams<F>,
    needs_final_permute_if_empty: bool,
    pub sponge_state: SpongeState<F>,
}

impl<F: FieldWitness> OptSponge<F> {
    pub fn create() -> Self {
        Self {
            state: [F::zero(); M],
            params: F::get_params(),
            needs_final_permute_if_empty: true,
            sponge_state: SpongeState::Absorbing {
                next_index: Boolean::False,
                xs: Vec::with_capacity(32),
            },
        }
    }

    pub fn absorb(&mut self, x: (CircuitVar<Boolean>, F)) {
        match &mut self.sponge_state {
            SpongeState::Absorbing { next_index: _, xs } => {
                xs.push(x);
            }
            SpongeState::Squeezed(_) => {
                self.sponge_state = SpongeState::Absorbing {
                    next_index: Boolean::False,
                    xs: {
                        let mut vec = Vec::with_capacity(32);
                        vec.push(x);
                        vec
                    },
                }
            }
        }
    }

    pub fn squeeze(&mut self, w: &mut Witness<F>) -> F {
        match &self.sponge_state {
            SpongeState::Squeezed(n) => {
                let n = *n;
                if n == RATE {
                    self.state = block_cipher(self.state, &self.params, w);
                    self.sponge_state = SpongeState::Squeezed(1);
                    self.state[0]
                } else {
                    self.sponge_state = SpongeState::Squeezed(n + 1);
                    self.state[n]
                }
            }
            SpongeState::Absorbing { next_index, xs } => {
                self.state = consume(
                    ConsumeParams {
                        needs_final_permute_if_empty: self.needs_final_permute_if_empty,
                        start_pos: CircuitVar::Constant(*next_index),
                        params: self.params,
                        input: &xs,
                        state: self.state,
                    },
                    w,
                );
                self.sponge_state = SpongeState::Squeezed(1);
                self.state[0]
            }
        }
    }
}

fn add_in<F: FieldWitness>(a: &mut [F; 3], i: Boolean, x: F, w: &mut Witness<F>) {
    let i_equals_0 = i.neg();
    let i_equals_1 = i;

    for (j, i_equals_j) in [i_equals_0, i_equals_1].iter().enumerate() {
        let a_j = w.exists({
            let a_j = a[j];
            match i_equals_j {
                Boolean::True => a_j + x,
                Boolean::False => a_j,
            }
        });
        a[j] = a_j;
    }
}

struct ConsumeParams<'a, F: FieldWitness> {
    needs_final_permute_if_empty: bool,
    start_pos: CircuitVar<Boolean>,
    params: &'static ArithmeticSpongeParams<F>,
    input: &'a [(CircuitVar<Boolean>, F)],
    state: [F; 3],
}

fn consume<F: FieldWitness>(params: ConsumeParams<F>, w: &mut Witness<F>) -> [F; 3] {
    let ConsumeParams {
        needs_final_permute_if_empty,
        start_pos,
        params,
        input,
        mut state,
    } = params;

    let mut pos = start_pos;

    let mut npermute = 0;

    let mut cond_permute =
        |permute: CircuitVar<Boolean>, state: &mut [F; M], w: &mut Witness<F>| {
            let permuted = block_cipher(*state, params, w);
            for (i, state) in state.iter_mut().enumerate() {
                let v = match permute.as_boolean() {
                    Boolean::True => permuted[i],
                    Boolean::False => *state,
                };
                if let CircuitVar::Var(_) = permute {
                    w.exists_no_check(v);
                }
                *state = v;
            }

            // for (i, state) in state.iter_mut().enumerate() {
            //     let v = match permute {
            //         Boolean::True => permuted[i],
            //         Boolean::False => *state,
            //     };
            //     if !(npermute > 2) {
            //         w.exists_no_check(v);
            //     }
            //     *state = v;
            // }

            npermute += 1;
            // state.(i) <- Field.if_ permute ~then_:permuted.(i) ~else_:state.(i)
        };

    let mut i = 0;

    // TODO: That's a mess, we need to implement cvars here
    let mut by_pairs = input.chunks_exact(2);
    while let Some(pairs) = by_pairs.next() {
        let (b, x) = pairs[0];
        let (b2, y) = pairs[1];

        let p = pos;
        let p2 = p.lxor(&b, w);
        pos = p2.lxor(&b2, w);

        let y = match b2 {
            CircuitVar::Var(b2) => field::mul(y, b2.to_field::<F>(), w),
            CircuitVar::Constant(b2) => y * b2.to_field::<F>(),
        };

        let add_in_y_after_perm = CircuitVar::all(&[b, b2, p], w);
        let add_in_y_before_perm = add_in_y_after_perm.map(Boolean::neg);

        // let add_in_y_after_perm =
        //     Boolean::all(&[b.as_boolean(), b2.as_boolean(), p.as_boolean()], w);
        // let add_in_y_before_perm = add_in_y_after_perm.neg();

        let product = match b {
            CircuitVar::Var(b) => field::mul(x, b.to_field::<F>(), w),
            CircuitVar::Constant(b) => x * b.to_field::<F>(),
        };
        add_in(&mut state, p.as_boolean(), product, w);

        let product = match add_in_y_before_perm {
            CircuitVar::Var(b) => field::mul(y, b.to_field(), w),
            CircuitVar::Constant(b) => y * b.to_field::<F>(),
        };
        add_in(
            &mut state,
            p2.as_boolean(),
            product,
            // field::mul(y, add_in_y_before_perm.as_boolean().to_field(), w),
            w,
        );

        let permute = {
            // We decompose this way because of OCaml evaluation order
            let b3 = CircuitVar::all(&[p, b.or(&b2, w)], w);
            let a = CircuitVar::all(&[b, b2], w);
            CircuitVar::any(&[a, b3], w)
        };

        cond_permute(permute, &mut state, w);

        let product = match add_in_y_after_perm {
            CircuitVar::Var(b) => field::mul(y, b.to_field(), w),
            CircuitVar::Constant(b) => y * b.to_field::<F>(),
        };

        add_in(
            &mut state,
            p2.as_boolean(),
            product,
            // field::mul(y, add_in_y_after_perm.as_boolean().to_field(), w),
            w,
        );

        i += 1;
    }

    let fst = |(f, _): &(CircuitVar<Boolean>, F)| *f;
    let fst_input = input.iter().map(fst).collect::<Vec<_>>();

    // Note: It's Boolean.Array.any here, not sure if there is a difference
    let empty_input = CircuitVar::any(&fst_input, w).map(Boolean::neg);

    let should_permute = match by_pairs.remainder() {
        &[] => {
            if needs_final_permute_if_empty {
                empty_input.or(&pos, w)
            } else {
                pos
            }
        }
        &[(b, x)] => {
            let p = pos;
            pos = p.lxor(&b, w);

            let product = match b {
                CircuitVar::Var(b) => field::mul(x, b.to_field::<F>(), w),
                CircuitVar::Constant(b) => x * b.to_field::<F>(),
            };

            add_in(&mut state, p.as_boolean(), product, w);

            if needs_final_permute_if_empty {
                CircuitVar::any(&[p, b, empty_input], w)
            } else {
                CircuitVar::any(&[p, b], w)
            }

            // let should_permute =
            //   match remaining with
            //   | 0 ->
            //       if needs_final_permute_if_empty then Boolean.(empty_imput ||| !pos)
            //       else !pos
            //   | 1 ->
            //       let b, x = input.(n - 1) in
            //       let p = !pos in
            //       pos := Boolean.( lxor ) p b ;
            //       add_in state p Field.(x * (b :> t)) ;
            //       if needs_final_permute_if_empty then Boolean.any [ p; b; empty_imput ]
            //       else Boolean.any [ p; b ]
            //   | _ ->
            //       assert false
            // in
            // cond_permute should_permute
        }
        _ => unreachable!(),
    };

    let _ = pos;
    cond_permute(should_permute, &mut state, w);

    state
}

fn block_cipher<F: FieldWitness>(
    mut state: [F; M],
    params: &ArithmeticSpongeParams<F>,
    w: &mut Witness<F>,
) -> [F; M] {
    w.exists(state);
    for r in 0..PERM_ROUNDS_FULL {
        full_round(&mut state, r, params, w);
    }
    state
}

fn full_round<F: FieldWitness>(
    state: &mut [F; M],
    r: usize,
    params: &ArithmeticSpongeParams<F>,
    w: &mut Witness<F>,
) {
    for state_i in state.iter_mut() {
        *state_i = sbox::<F>(*state_i);
    }
    *state = apply_mds_matrix::<F>(params, &state);
    for (i, x) in params.round_constants[r].iter().enumerate() {
        state[i].add_assign(x);
    }
    w.exists(*state);
}

fn sbox<F: FieldWitness>(x: F) -> F {
    let mut res = x.square();
    res *= x;
    let res = res.square();
    res * x
}

fn apply_mds_matrix<F: FieldWitness>(params: &ArithmeticSpongeParams<F>, state: &[F; 3]) -> [F; 3] {
    std::array::from_fn(|i| {
        state
            .iter()
            .zip(params.mds[i].iter())
            .fold(F::zero(), |x, (s, &m)| m * s + x)
    })
}

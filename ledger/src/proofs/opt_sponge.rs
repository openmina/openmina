use crate::ArithmeticSpongeParams;

use super::{
    field::{field, Boolean, CircuitVar, FieldWitness},
    witness::Witness,
};

const M: usize = 3;
const CAPACITY: usize = 1;
const RATE: usize = M - CAPACITY;
const PERM_ROUNDS_FULL: usize = 55;

// REVIEW(dw): why not using proof-system code instead directly for the sponge?
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

    pub fn of_sponge(sponge: super::transaction::poseidon::Sponge<F>, w: &mut Witness<F>) -> Self {
        use super::transaction::poseidon::Sponge;

        let Sponge {
            sponge_state,
            state,
            ..
        } = sponge;

        match sponge_state {
            mina_poseidon::poseidon::SpongeState::Squeezed(n) => Self {
                state,
                params: F::get_params(),
                needs_final_permute_if_empty: true,
                sponge_state: SpongeState::Squeezed(n),
            },
            mina_poseidon::poseidon::SpongeState::Absorbed(n) => {
                let abs = |i: Boolean| Self {
                    state,
                    params: F::get_params(),
                    needs_final_permute_if_empty: true,
                    sponge_state: SpongeState::Absorbing {
                        next_index: i,
                        xs: vec![],
                    },
                };

                match n {
                    0 => abs(Boolean::False),
                    1 => abs(Boolean::True),
                    2 => Self {
                        state: { block_cipher(state, F::get_params(), w) },
                        params: F::get_params(),
                        needs_final_permute_if_empty: false,
                        sponge_state: SpongeState::Absorbing {
                            next_index: Boolean::False,
                            xs: vec![],
                        },
                    },
                    _ => panic!(),
                }
            }
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
                    self.state = block_cipher(self.state, self.params, w);
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
                        input: xs,
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

// REVIEW(dw): should it be here?
fn add_in<F: FieldWitness>(a: &mut [F; 3], i: CircuitVar<Boolean>, x: F, w: &mut Witness<F>) {
    let i = i.as_boolean();
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

// REVIEW(dw): should it be here?
fn mul_by_boolean<F>(x: F, y: CircuitVar<Boolean>, w: &mut Witness<F>) -> F
where
    F: FieldWitness,
{
    match y {
        CircuitVar::Var(y) => field::mul(x, y.to_field::<F>(), w),
        CircuitVar::Constant(y) => x * y.to_field::<F>(),
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

            npermute += 1;
        };

    let mut by_pairs = input.chunks_exact(2);
    for pairs in by_pairs.by_ref() {
        let (b, x) = pairs[0];
        let (b2, y) = pairs[1];

        let p = pos;
        let p2 = p.lxor(&b, w);
        pos = p2.lxor(&b2, w);

        let y = mul_by_boolean(y, b2, w);

        let add_in_y_after_perm = CircuitVar::all(&[b, b2, p], w);
        let add_in_y_before_perm = add_in_y_after_perm.neg();

        let product = mul_by_boolean(x, b, w);
        add_in(&mut state, p, product, w);

        let product = mul_by_boolean(y, add_in_y_before_perm, w);
        add_in(&mut state, p2, product, w);

        let permute = {
            // We decompose this way because of OCaml evaluation order
            let b3 = CircuitVar::all(&[p, b.or(&b2, w)], w);
            let a = CircuitVar::all(&[b, b2], w);
            CircuitVar::any(&[a, b3], w)
        };

        cond_permute(permute, &mut state, w);

        let product = mul_by_boolean(y, add_in_y_after_perm, w);
        add_in(&mut state, p2, product, w);
    }

    let fst = |(f, _): &(CircuitVar<Boolean>, F)| *f;
    let fst_input = input.iter().map(fst).collect::<Vec<_>>();

    // Note: It's Boolean.Array.any here, not sure if there is a difference
    let empty_input = CircuitVar::any(&fst_input, w).map(Boolean::neg);

    let should_permute = match *by_pairs.remainder() {
        [] => {
            if needs_final_permute_if_empty {
                empty_input.or(&pos, w)
            } else {
                pos
            }
        }
        [(b, x)] => {
            let p = pos;
            pos = p.lxor(&b, w);

            let product = mul_by_boolean(x, b, w);
            add_in(&mut state, p, product, w);

            if needs_final_permute_if_empty {
                CircuitVar::any(&[p, b, empty_input], w)
            } else {
                CircuitVar::any(&[p, b], w)
            }
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
    *state = apply_mds_matrix::<F>(params, state);
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

use std::marker::PhantomData;

use ark_ff::{BigInteger256, Field};
use mina_curves::pasta::{Fp, Fq};

mod params;

pub use params::*;

pub trait SpongeConstants {
    const SPONGE_CAPACITY: usize = 1;
    const SPONGE_WIDTH: usize = 3;
    const SPONGE_RATE: usize = 2;
    const PERM_ROUNDS_FULL: usize;
    const PERM_ROUNDS_PARTIAL: usize;
    const PERM_HALF_ROUNDS_FULL: usize;
    const PERM_SBOX: u32;
    const PERM_FULL_MDS: bool;
    const PERM_INITIAL_ARK: bool;
}

#[derive(Clone)]
pub struct PlonkSpongeConstantsKimchi {}

impl SpongeConstants for PlonkSpongeConstantsKimchi {
    const SPONGE_CAPACITY: usize = 1;
    const SPONGE_WIDTH: usize = 3;
    const SPONGE_RATE: usize = 2;
    const PERM_ROUNDS_FULL: usize = 55;
    const PERM_ROUNDS_PARTIAL: usize = 0;
    const PERM_HALF_ROUNDS_FULL: usize = 0;
    const PERM_SBOX: u32 = 7;
    const PERM_FULL_MDS: bool = true;
    const PERM_INITIAL_ARK: bool = false;
}

#[derive(Clone)]
pub struct PlonkSpongeConstantsLegacy {}

impl SpongeConstants for PlonkSpongeConstantsLegacy {
    const SPONGE_CAPACITY: usize = 1;
    const SPONGE_WIDTH: usize = 3;
    const SPONGE_RATE: usize = 2;
    const PERM_ROUNDS_FULL: usize = 63;
    const PERM_ROUNDS_PARTIAL: usize = 0;
    const PERM_HALF_ROUNDS_FULL: usize = 0;
    const PERM_SBOX: u32 = 5;
    const PERM_FULL_MDS: bool = true;
    const PERM_INITIAL_ARK: bool = true;
}

fn apply_mds_matrix<F: Field, SC: SpongeConstants>(
    params: &ArithmeticSpongeParams<F>,
    state: &[F],
) -> [F; 3] {
    let mut new_state = [F::zero(); 3];

    for (i, sub_params) in params.mds.iter().enumerate() {
        for (state, param) in state.iter().zip(sub_params) {
            new_state[i].add_assign(*param * state);
        }
    }

    new_state
}

pub fn full_round<F: Field, SC: SpongeConstants>(
    params: &ArithmeticSpongeParams<F>,
    state: &mut [F; 3],
    r: usize,
) {
    for state_i in state.iter_mut() {
        *state_i = sbox::<F, SC>(*state_i);
    }
    *state = apply_mds_matrix::<F, SC>(params, state);
    for (i, x) in params.round_constants[r].iter().enumerate() {
        state[i].add_assign(x);
    }
}

pub fn poseidon_block_cipher<F: Field, SC: SpongeConstants>(
    params: &ArithmeticSpongeParams<F>,
    state: &mut [F; 3],
) {
    if SC::PERM_INITIAL_ARK {
        for (i, x) in params.round_constants[0].iter().enumerate() {
            state[i].add_assign(x);
        }
        for r in 0..SC::PERM_ROUNDS_FULL {
            full_round::<F, SC>(params, state, r + 1);
        }
    } else {
        for r in 0..SC::PERM_ROUNDS_FULL {
            full_round::<F, SC>(params, state, r);
        }
    }
}

pub fn sbox<F: Field, SC: SpongeConstants>(mut x: F) -> F {
    // Faster than calling x.pow(SC::PERM_SBOX)

    if SC::PERM_SBOX == 7 {
        let mut res = x.square();
        res *= x;
        let res = res.square();
        res * x
    } else {
        let a = x;
        for _ in 0..SC::PERM_SBOX - 1 {
            x.mul_assign(a);
        }
        x
    }
    // x.pow([SC::PERM_SBOX as u64])
}

#[derive(Clone, Debug)]
pub enum SpongeState {
    Absorbed(usize),
    Squeezed(usize),
}

#[derive(Debug)]
pub struct ArithmeticSpongeParams<F: Field> {
    pub round_constants: Box<[[F; 3]]>,
    pub mds: [[F; 3]; 3],
}

pub trait SpongeParamsForField<F: Field> {
    fn get_params() -> &'static ArithmeticSpongeParams<F>;
}

impl SpongeParamsForField<Fp> for Fp {
    fn get_params() -> &'static ArithmeticSpongeParams<Fp> {
        fp::params()
    }
}

impl SpongeParamsForField<Fq> for Fq {
    fn get_params() -> &'static ArithmeticSpongeParams<Fq> {
        fq::params()
    }
}

#[derive(Clone)]
pub struct ArithmeticSponge<F: Field, C: SpongeConstants = PlonkSpongeConstantsKimchi> {
    pub sponge_state: SpongeState,
    rate: usize,
    pub state: [F; 3],
    params: &'static ArithmeticSpongeParams<F>,
    constants: PhantomData<C>,
}

impl<F: Field, C: SpongeConstants> ArithmeticSponge<F, C> {
    pub fn full_round(&mut self, r: usize) {
        full_round::<F, C>(self.params, &mut self.state, r);
    }

    fn poseidon_block_cipher(&mut self) {
        poseidon_block_cipher::<F, C>(self.params, &mut self.state);
    }
}

impl<F: Field + SpongeParamsForField<F>, C: SpongeConstants> ArithmeticSponge<F, C> {
    pub fn new_with_params(params: &'static ArithmeticSpongeParams<F>) -> ArithmeticSponge<F, C> {
        ArithmeticSponge {
            state: [F::zero(); 3],
            rate: C::SPONGE_RATE,
            sponge_state: SpongeState::Absorbed(0),
            params,
            constants: PhantomData,
        }
    }

    pub fn new() -> ArithmeticSponge<F, C> {
        Self::new_with_params(F::get_params())
    }

    pub fn absorb(&mut self, x: &[F]) {
        if x.is_empty() {
            // Same as the loop below but doesn't add `x`
            match self.sponge_state {
                SpongeState::Absorbed(n) => {
                    if n == self.rate {
                        self.poseidon_block_cipher();
                        self.sponge_state = SpongeState::Absorbed(1);
                    } else {
                        self.sponge_state = SpongeState::Absorbed(n + 1);
                    }
                }
                SpongeState::Squeezed(_n) => {
                    self.sponge_state = SpongeState::Absorbed(1);
                }
            }
            return;
        }
        for x in x.iter() {
            match self.sponge_state {
                SpongeState::Absorbed(n) => {
                    if n == self.rate {
                        self.poseidon_block_cipher();
                        self.sponge_state = SpongeState::Absorbed(1);
                        self.state[0].add_assign(x);
                    } else {
                        self.sponge_state = SpongeState::Absorbed(n + 1);
                        self.state[n].add_assign(x);
                    }
                }
                SpongeState::Squeezed(_n) => {
                    self.state[0].add_assign(x);
                    self.sponge_state = SpongeState::Absorbed(1);
                }
            }
        }
    }

    pub fn squeeze(&mut self) -> F {
        // assert_eq!(self.state.len(), 3);
        // elog!("NSTATE={:?}", self.state.len());
        match self.sponge_state {
            SpongeState::Squeezed(n) => {
                if n == self.rate {
                    self.poseidon_block_cipher();
                    self.sponge_state = SpongeState::Squeezed(1);
                    self.state[0]
                } else {
                    self.sponge_state = SpongeState::Squeezed(n + 1);
                    self.state[n]
                }
            }
            SpongeState::Absorbed(_n) => {
                self.poseidon_block_cipher();
                self.sponge_state = SpongeState::Squeezed(1);
                self.state[0]
            }
        }
    }
}

#[derive(Clone)]
pub struct FqSponge<F: Field> {
    sponge: ArithmeticSponge<F>,
    // sponge: ArithmeticSponge<F, PlonkSpongeConstantsKimchi, 55>,
    last_squeezed: Vec<u64>,
}

impl<F: Field + SpongeParamsForField<F> + Into<BigInteger256>> FqSponge<F> {
    pub fn new() -> Self {
        Self {
            sponge: ArithmeticSponge::new(),
            last_squeezed: Vec::with_capacity(8),
        }
    }

    pub fn absorb_fq(&mut self, x: &[F]) {
        self.last_squeezed.clear();
        for fe in x {
            self.sponge.absorb(&[*fe])
        }
    }

    pub fn squeeze_limbs<const NUM_LIMBS: usize>(&mut self) -> [u64; NUM_LIMBS] {
        const HIGH_ENTROPY_LIMBS: usize = 2;

        if self.last_squeezed.len() >= NUM_LIMBS {
            let limbs = std::array::from_fn(|i| self.last_squeezed[i]);

            let nremains = self.last_squeezed.len() - NUM_LIMBS;
            self.last_squeezed.copy_within(NUM_LIMBS.., 0);
            self.last_squeezed.truncate(nremains);

            limbs
        } else {
            let x: BigInteger256 = self.sponge.squeeze().into();
            self.last_squeezed
                .extend(&x.as_ref()[0..HIGH_ENTROPY_LIMBS]);
            self.squeeze_limbs::<NUM_LIMBS>()
        }
    }
}

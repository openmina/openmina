use ark_ff::One;
use mina_hasher::Fp;
use std::ops::{Add, Mul, Sub};

use crate::EvalsInCircuit;

pub struct ScalarsEnv {
    pub zk_polynomial: Fp,
    pub zeta_to_n_minus_1: Fp,
    pub srs_length_log2: u64,
}

pub enum CurrOrNext {
    Curr,
    Next,
}

pub enum Column {
    Witness(usize),
    Poseidon,
}

fn fst(array: &[Fp; 2]) -> Fp {
    array[0]
}

fn snd(array: &[Fp; 2]) -> Fp {
    array[1]
}

fn square(field: Fp) -> Fp {
    field * field
}

pub struct ScalarsEnv2 {
    pub zk_polynomial: Fp,
    pub zeta_to_n_minus_1: Fp,
    pub srs_length_log2: u64,
    pub var: Box<dyn Fn(Column, CurrOrNext) -> Fp>,
    pub square: fn(Fp) -> Fp,
    pub add: fn(Fp, Fp) -> Fp,
    pub sub: fn(Fp, Fp) -> Fp,
    pub mul: fn(Fp, Fp) -> Fp,
    pub field: fn(&str) -> Fp,
    pub alpha_pow: fn(n: usize) -> Fp,
}

pub enum Index {
    Poseidon,
    CompleteAdd,
    VarBaseMul,
    EndoMul,
    EndoMulScalar,
}

// type t = Poseidon | VarBaseMul | EndoMul | CompleteAdd | EndoMulScalar

impl ScalarsEnv2 {
    pub fn new(circuit: EvalsInCircuit) -> Self {
        let w0 = circuit.w.map(|w| w[0]);
        let w1 = circuit.w.map(|w| w[1]);

        let var = move |col: Column, row: CurrOrNext| {
            // Cast them to the same type, or the compiler will complain
            let fst = fst as fn(array: &[Fp; 2]) -> Fp;
            let snd = snd as fn(array: &[Fp; 2]) -> Fp;

            let (get_eval, w) = match row {
                CurrOrNext::Curr => (fst, &w0),
                CurrOrNext::Next => (snd, &w1),
            };

            match col {
                Column::Witness(i) => w[i],
                Column::Poseidon => get_eval(&circuit.poseidon_selector),
            }
        };

        ScalarsEnv2 {
            zk_polynomial: Fp::one(),
            zeta_to_n_minus_1: Fp::one(),
            srs_length_log2: 1,
            var: Box::new(var),
            square,
            add: Fp::add,
            sub: Fp::sub,
            mul: Fp::mul,
            field: todo!(),
            alpha_pow: todo!(),
        }
    }

    pub fn lookup(&self, index: Index) {
        use Column::*;
        use CurrOrNext::*;

        match index {
            Index::CompleteAdd => {
                let x0 =
                    (self.var)(Column::Witness(2), Curr) - (self.var)(Column::Witness(0), Curr);
                let x1 =
                    (self.var)(Column::Witness(3), Curr) - (self.var)(Column::Witness(1), Curr);
                let x2 =
                    (self.var)(Column::Witness(0), Curr) * (self.var)(Column::Witness(0), Curr);

                ((self.var)(Column::Witness(10), Curr) * x0)
                    - ((self.field)(
                        "0x0000000000000000000000000000000000000000000000000000000000000001",
                    ) - (self.var)(Column::Witness(7), Curr))
                    + ((self.alpha_pow)(1) * (self.var)(Column::Witness(7), Curr) * x0)
                    + ((self.alpha_pow)(2) * ((self.var)(Column::Witness(7), Curr) * x0));
            }
            Index::Poseidon => todo!(),
            Index::VarBaseMul => todo!(),
            Index::EndoMul => todo!(),
            Index::EndoMulScalar => todo!(),
        }
    }
}

//  + alpha_pow 2
//    * ( cell (var (Witness 7, Curr))
//        * ( double (cell (var (Witness 8, Curr)))
//            * cell (var (Witness 1, Curr))
//          - double x_2 - x_2 )
//      + ( field
//            "0x0000000000000000000000000000000000000000000000000000000000000001"
//        - cell (var (Witness 7, Curr)) )
//        * ((x_0 * cell (var (Witness 8, Curr))) - x_1) )
//  + alpha_pow 3
//    * ( cell (var (Witness 0, Curr))
//      + cell (var (Witness 2, Curr))
//      + cell (var (Witness 4, Curr))
//      - (cell (var (Witness 8, Curr)) * cell (var (Witness 8, Curr)))
//      )
//  + alpha_pow 4
//    * ( cell (var (Witness 8, Curr))
//        * ( cell (var (Witness 0, Curr))
//          - cell (var (Witness 4, Curr)) )
//      - cell (var (Witness 1, Curr))
//      - cell (var (Witness 5, Curr)) )
//  + alpha_pow 5
//    * ( x_1
//      * (cell (var (Witness 7, Curr)) - cell (var (Witness 6, Curr)))
//      )
//  + alpha_pow 6
//    * ( (x_1 * cell (var (Witness 9, Curr)))
//      - cell (var (Witness 6, Curr)) ) ) )

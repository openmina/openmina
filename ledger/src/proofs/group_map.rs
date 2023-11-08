use std::ops::Neg;

use crate::proofs::witness::{field, FieldWitness, Witness};

pub mod bw19 {
    use super::*;

    struct Spec<F: FieldWitness> {
        b: F,
    }

    #[derive(Clone, Copy)]
    pub struct Params<F: FieldWitness> {
        u: F,
        fu: F,
        sqrt_neg_three_u_squared_minus_u_over_2: F,
        sqrt_neg_three_u_squared: F,
        inv_three_u_squared: F,
        b: F,
    }

    impl<F: FieldWitness> Params<F> {
        fn create_impl() -> Self {
            let b = F::PARAMS.b;

            fn first_map<F: FieldWitness, T>(f: impl Fn(F) -> Option<T>) -> T {
                let mut v = F::zero();
                let one = F::one();
                loop {
                    match f(v) {
                        Some(x) => return x,
                        None => v = v + one,
                    }
                }
            }

            let curve_eqn = |u: F| (u * u * u) + b;

            let (u, fu) = first_map(|u| {
                let fu = curve_eqn(u);
                if u.is_zero() || fu.is_zero() {
                    None
                } else {
                    Some((u, fu))
                }
            });

            let three_u_squared = u * u * F::from(3);
            let sqrt_neg_three_u_squared = three_u_squared.neg().sqrt().unwrap();

            Self {
                u,
                fu,
                sqrt_neg_three_u_squared_minus_u_over_2: (sqrt_neg_three_u_squared - u)
                    / F::from(2),
                sqrt_neg_three_u_squared,
                inv_three_u_squared: F::one() / three_u_squared,
                b,
            }
        }

        pub fn create() -> Self {
            cache!(Self, Self::create_impl())
        }
    }

    pub fn potential_xs<F: FieldWitness>(
        t: F,
        params: &Params<F>,
        w: &mut Witness<F>,
    ) -> (F, F, F) {
        let square = |x: F, w: &mut Witness<F>| field::mul(x, x, w);

        let t2 = field::mul(t, t, w);
        let alpha = {
            let alpha_inv = field::mul(t2 + params.fu, t2, w);
            field::div(F::one(), alpha_inv, w)
        };
        let x1 = {
            let t2_squared = square(t2, w);
            let temp = field::mul(t2_squared, alpha, w) * params.sqrt_neg_three_u_squared;
            params.sqrt_neg_three_u_squared_minus_u_over_2 - temp
        };
        let x2 = params.u.neg() - x1;
        let x3 = {
            let t2_plus_fu = t2 + params.fu;
            let t2_inv = field::mul(alpha, t2_plus_fu, w);
            let temp = {
                let t2_plus_fu_squared = square(t2_plus_fu, w);
                field::mul(t2_plus_fu_squared, t2_inv, w) * params.inv_three_u_squared
            };
            params.u - temp
        };
        (x1, x2, x3)
    }
}

use ark_ff::{FpParameters, One};
use mina_hasher::Fp;

use self::tock::Conic;

use super::witness::{make_group, Boolean, GroupAffine, ToBoolean};

fn sqrt_exn<F: FieldWitness>(x: F, w: &mut Witness<F>) -> F {
    let y = w.exists(x.sqrt().unwrap());
    y
}

fn is_square<F: FieldWitness>(x: F) -> bool {
    let s = x.pow(F::Params::MODULUS_MINUS_ONE_DIV_TWO);
    s.is_zero() || s.is_one()
}

fn sqrt_flagged<F: FieldWitness>(x: F, w: &mut Witness<F>) -> (F, Boolean) {
    let is_square = w.exists(is_square(x).to_boolean());
    let m = non_residue::<F>();
    let v = w.exists_no_check(match is_square {
        Boolean::True => x,
        Boolean::False => x * m,
    });
    (sqrt_exn(v, w), is_square)
}

fn non_residue<F: FieldWitness>() -> F {
    cache!(F, {
        let mut i = F::from(2);
        let one = F::one();
        loop {
            if !is_square(i) {
                break i;
            } else {
                i += one;
            }
        }
    })
}

pub fn bw19_wrap<F: FieldWitness>(
    x: F,
    params: &bw19::Params<F>,
    w: &mut Witness<F>,
) -> GroupAffine<F> {
    let potential_xs = bw19::potential_xs(x, params, w);
    wrap(potential_xs, w)
}

pub fn wrap<F: FieldWitness>(potential_xs: (F, F, F), w: &mut Witness<F>) -> GroupAffine<F> {
    let y_squared =
        |x: F, w: &mut Witness<F>| field::muls(&[x, x, x], w) + (F::PARAMS.a * x) + F::PARAMS.b;

    let (x1, x2, x3) = potential_xs;

    let (y1, b1) = sqrt_flagged(y_squared(x1, w), w);
    let (y2, b2) = sqrt_flagged(y_squared(x2, w), w);
    let (y3, b3) = sqrt_flagged(y_squared(x3, w), w);

    Boolean::assert_any(&[b1, b2, b3], w);

    let x1_is_first = b1.to_field::<F>();
    let x2_is_first = (b1.neg().and(&b2, w)).to_field::<F>();
    let x3_is_first = (b1.neg().and(&b2.neg(), w).and(&b3, w)).to_field::<F>();

    // We decompose this way because of OCaml evaluation order
    let x3_is_first_y3 = field::mul(x3_is_first, y3, w);
    let x2_is_first_y2 = field::mul(x2_is_first, y2, w);
    let x1_is_first_y1 = field::mul(x1_is_first, y1, w);
    let x3_is_first_x3 = field::mul(x3_is_first, x3, w);
    let x2_is_first_x2 = field::mul(x2_is_first, x2, w);
    let x1_is_first_x1 = field::mul(x1_is_first, x1, w);

    let (x, y) = (
        (x1_is_first_x1 + x2_is_first_x2 + x3_is_first_x3),
        (x1_is_first_y1 + x2_is_first_y2 + x3_is_first_y3),
    );
    make_group(x, y)
}

mod tock {
    use super::*;
    use std::ops::Neg;

    use ark_ff::{SquareRootField, Zero};
    use mina_hasher::Fp;

    /// A good name from OCaml
    #[derive(Clone, Debug)]
    pub struct S<F: FieldWitness> {
        pub u: F,
        pub v: F,
        pub y: F,
    }

    #[derive(Clone, Debug)]
    pub struct Conic<F: FieldWitness> {
        pub z: F,
        pub y: F,
    }

    #[derive(Clone, Debug)]
    pub struct Spec<F> {
        pub a: F,
        pub b: F,
    }

    #[derive(Clone, Debug)]
    pub struct Params<F: FieldWitness> {
        pub u: F,
        pub u_over_2: F,
        pub projection_point: Conic<F>,
        pub conic_c: F,
        pub spec: Spec<F>,
    }

    fn tock_params_impl() -> Params<Fp> {
        let a = Fp::PARAMS.a;
        let b = Fp::PARAMS.b;

        fn first_map<F: FieldWitness, T>(f: impl Fn(F) -> Option<T>) -> T {
            let mut v = F::zero();
            let one = F::one();
            loop {
                match f(v) {
                    Some(x) => return x,
                    None => v = v + one,
                }
            }
        }

        fn first<F: FieldWitness>(f: impl Fn(F) -> bool) -> F {
            first_map(|x| match f(x) {
                true => Some(x),
                false => None,
            })
        }

        let three_fourths = Fp::from(3) / Fp::from(4);
        let curve_eqn = |u: Fp| (u * u * u) + (a * u) + b;

        let u = first(|u: Fp| {
            let check = (three_fourths * u * u) + a;
            let fu = curve_eqn(u);

            !(check.is_zero()) && !(fu.is_zero()) && !(is_square(fu.neg()))
        });

        let conic_c = (three_fourths * u * u) + a;
        let conic_d = curve_eqn(u).neg();

        let projection_point = first_map(|y: Fp| {
            let z2 = conic_d - (conic_c * y * y);

            if is_square(z2) {
                Some(Conic {
                    z: z2.sqrt().unwrap(),
                    y,
                })
            } else {
                None
            }
        });

        Params {
            u,
            u_over_2: u / Fp::from(2),
            projection_point,
            conic_c,
            spec: Spec { a, b },
        }
    }

    pub fn params() -> Params<Fp> {
        cache_one!(Params<Fp>, { tock_params_impl() })
    }
}

fn field_to_conic(t: Fp, params: &tock::Params<Fp>, w: &mut Witness<Fp>) -> tock::Conic<Fp> {
    let tock::Conic { z: z0, y: y0 } = params.projection_point;

    let one = Fp::one();

    let ct = params.conic_c * t;
    let s = field::div_by_inv(
        Fp::from(2) * ((ct * y0) + z0),
        field::mul(ct, t, w) + one,
        w,
    );

    tock::Conic {
        z: z0 - s,
        y: y0 - field::mul(s, t, w),
    }
}

fn conic_to_s(p: Conic<Fp>, params: &tock::Params<Fp>, w: &mut Witness<Fp>) -> tock::S<Fp> {
    let Conic { z, y } = p;
    let u = params.u;
    let v = field::div_by_inv(z, y, w) - params.u_over_2;
    tock::S { u, v, y }
}

fn s_to_v_truncated(s: tock::S<Fp>, w: &mut Witness<Fp>) -> (Fp, Fp, Fp) {
    let tock::S { u, v, y } = s;

    (v, (u + v).neg(), u + field::mul(y, y, w))
}

pub fn to_group(t: Fp, w: &mut Witness<Fp>) -> GroupAffine<Fp> {
    let params = tock::params();

    let potential_xs = {
        let conic = field_to_conic(t, &params, w);
        let s = conic_to_s(conic, &params, w);
        s_to_v_truncated(s, w)
    };

    wrap(potential_xs, w)
}

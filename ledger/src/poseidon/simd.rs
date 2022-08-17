#[cfg(test)]
mod tests {
    use std::{
        ops::{Add, Mul},
        str::FromStr,
    };

    use ark_ff::{BigInteger256, One, PrimeField};
    use mina_hasher::Fp;
    use packed_simd::{f64x4, u64x4, u64x8};

    // use super::*;

    #[test]
    fn test_basic() {
        let zero: u64x8 = u64x8::new(0, 0, 0, 0, 0, 0, 0, 0);
        println!("zero={:?}", zero);
        println!(
            "one ={:?}",
            u64x8::new(u64::MAX, 0, 0, 0, 0, 0, 0, 0) + u64x8::new(2, 0, 0, 0, 0, 0, 0, 0)
        );
        println!("ten ={:?}", zero.add(1).mul(10));

        let fp = Fp::from_str(
            "12035446894107573964500871153637039653510326950134440362813193268448863222019",
        )
        .unwrap();
        let bigint: BigInteger256 = fp.into();
        let bigref = bigint.as_ref();

        // let one = Fp::one();
        // println!("one={:?}", one);
        // println!("one={:?}", one.into_repr());

        assert_eq!(bigref.len(), 4);

        let simd: u64x4 = u64x4::from_slice_unaligned(bigref);

        let abc = [bigref[0], bigref[1], bigref[2], bigref[3], 0, 0, 0, 0];
        // let abc = [0, 0, 0, 0, bigref[0], bigref[1], bigref[2], bigref[3]];
        let simd2: u64x8 = u64x8::from_slice_unaligned(&abc[..]);
        // let fsimd: f64x4 = f64x4::from_slice_unaligned(&bigref.iter().map(|f| *f as f64).collect::<Vec<_>>());

        println!("simd  ={:?}", simd);
        println!("simd2  ={:?}", simd2);
        // println!("fsimd  ={:?}", simd);
        println!("fp    ={:?}", fp);
        println!("bigint={:?}", bigint);
        println!("ref   ={:?}\n", bigref);

        let n = 2;
        let simd = simd * n;
        let simd2 = simd2 * n;
        // let fsimd = fsimd * (n as f64);
        let fp: Fp = fp.mul(Fp::from(n));

        let simd2 = simd2
            % u64x8::new(
                // 0xcc96987680000000,
                // 0x11234c7e04a67c8d,
                // 0x0,
                // 0x2000000000000000,
                0x992d30ed00000001,
                0x224698fc094cf91b,
                0x0,
                0x4000000000000000,
                0,
                0,
                0,
                0,
            );

        // let simd2 = simd % 3;

        println!("simd  ={:?}", simd);
        println!("simd2 ={:?}", simd2);
        // // println!("fsimd  ={:?}", fsimd);
        println!("fp    ={:?}", fp);
        println!("bigint={:?}", fp.0);
        let bigint: BigInteger256 = fp.into();
        println!("bigint2={:?}", bigint);
        // let bigint3 = BigInteger256::new([simd.extract(0), simd.extract(1), simd.extract(2), simd.extract(3)]);
        // let fp2: Fp = bigint3.into();
        // println!("bigint3={:?}", bigint3);
        // println!("fp2={:?}", fp2);
        // // Fp::read(&[simd.extract(0), simd.extract(1), simd.extract(2), simd.extract(3)][..]);

        // let acc = Account::create();
        // let hash = acc.hash();
    }
}

// impl FftParameters for FpParameters {
//     type BigInt = BigInteger;

//     const TWO_ADICITY: u32 = 32;

//     #[rustfmt::skip]
//     const TWO_ADIC_ROOT_OF_UNITY: BigInteger = BigInteger([
//         0xa28db849bad6dbf0, 0x9083cd03d3b539df, 0xfba6b9ca9dc8448e, 0x3ec928747b89c6da
//     ]);
// }

// impl ark_ff::FpParameters for FpParameters {
//     // 28948022309329048855892746252171976963363056481941560715954676764349967630337
//     const MODULUS: BigInteger = BigInteger([
//         0x992d30ed00000001,
//         0x224698fc094cf91b,
//         0x0,
//         0x4000000000000000,
//     ]);

//     const R: BigInteger = BigInteger([
//         0x34786d38fffffffd,
//         0x992c350be41914ad,
//         0xffffffffffffffff,
//         0x3fffffffffffffff,
//     ]);

//     const R2: BigInteger = BigInteger([
//         0x8c78ecb30000000f,
//         0xd7d30dbd8b0de0e7,
//         0x7797a99bc3c95d18,
//         0x96d41af7b9cb714,
//     ]);

//     const MODULUS_MINUS_ONE_DIV_TWO: BigInteger = BigInteger([
//         0xcc96987680000000,
//         0x11234c7e04a67c8d,
//         0x0,
//         0x2000000000000000,
//     ]);

//     // T and T_MINUS_ONE_DIV_TWO, where MODULUS - 1 = 2^S * T
//     const T: BigInteger = BigInteger([0x94cf91b992d30ed, 0x224698fc, 0x0, 0x40000000]);

//     const T_MINUS_ONE_DIV_TWO: BigInteger =
//         BigInteger([0x4a67c8dcc969876, 0x11234c7e, 0x0, 0x20000000]);

//     // GENERATOR = 5
//     const GENERATOR: BigInteger = BigInteger([
//         0xa1a55e68ffffffed,
//         0x74c2a54b4f4982f3,
//         0xfffffffffffffffd,
//         0x3fffffffffffffff,
//     ]);

//     const MODULUS_BITS: u32 = 255;

//     const CAPACITY: u32 = Self::MODULUS_BITS - 1;

//     const REPR_SHAVE_BITS: u32 = 1;

//     // -(MODULUS^{-1} mod 2^64) mod 2^64
//     const INV: u64 = 11037532056220336127;
// }

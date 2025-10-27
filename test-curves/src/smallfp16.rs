use ark_ff::ark_ff_macros::SmallFpConfig;
use ark_ff::{BigInt, SmallFp, SmallFpConfig, SqrtPrecomputation};

#[derive(SmallFpConfig)]
#[modulus = "65521"]
#[generator = "17"]
#[backend = "standard"]
pub struct SmallF16Config;
pub type SmallF16 = SmallFp<SmallF16Config>;

#[derive(SmallFpConfig)]
#[modulus = "65521"]
#[generator = "17"]
#[backend = "montgomery"]
pub struct SmallF16ConfigMont;
pub type SmallF16Mont = SmallFp<SmallF16ConfigMont>;

#[derive(SmallFpConfig)]
#[modulus = "65521"]
#[generator = "17"]
#[backend = "standard"]
#[simd = "true"]
pub struct SmallF16SimdConfig;
pub type SmallF16Simd = SmallFp<SmallF16SimdConfig>;

#[cfg(test)]
mod tests {
    use super::*;
    use ark_algebra_test_templates::*;
    use ark_std::vec;

    test_small_field!(f16; SmallF16);
    test_small_field!(f16_mont; SmallF16Mont);

    // Temporarily disabled due to missing SmallFpSimd trait methods
    /*
    use ark_ff::{PrimeField, SmallFpSimd, UniformRand, Zero};

    const SIMD_TEST_SIZE: usize = 10_000;
    #[test]
    fn simd_add_assign_works() {
        let mut rng = ark_std::test_rng();
        let a: vec::Vec<SmallF16> = (0..SIMD_TEST_SIZE)
            .map(|_| SmallF16::rand(&mut rng))
            .collect();
        let b: vec::Vec<SmallF16> = (0..SIMD_TEST_SIZE)
            .map(|_| SmallF16::rand(&mut rng))
            .collect();

        let mut a_simd: vec::Vec<SmallF16Simd> = a.iter().map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap()).collect();
        let b_simd: vec::Vec<SmallF16Simd> = b.iter().map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap()).collect();

        let mut expected = a.clone();
        for (val_a, val_b) in expected.iter_mut().zip(b.iter()) {
            *val_a += val_b;
        }

        SmallF16SimdConfig::add_assign_simd(&mut a_simd, &b_simd);
        for i in 0..SIMD_TEST_SIZE {
            assert_eq!(a_simd[i].into_bigint(), expected[i].into_bigint());
        }
    }

    #[test]
    fn simd_mul_assign_works() {
        let mut rng = ark_std::test_rng();
        let a: vec::Vec<SmallF16> = (0..SIMD_TEST_SIZE).map(|_| SmallF16::rand(&mut rng)).collect();
        let b: vec::Vec<SmallF16> = (0..SIMD_TEST_SIZE).map(|_| SmallF16::rand(&mut rng)).collect();

        let mut a_simd: vec::Vec<SmallF16Simd> = a.iter().map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap()).collect();
        let b_simd: vec::Vec<SmallF16Simd> = b.iter().map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap()).collect();

        let mut expected = a.clone();
        for (val_a, val_b) in expected.iter_mut().zip(b.iter()) { *val_a *= val_b; }

        SmallF16SimdConfig::mul_assign_simd(&mut a_simd, &b_simd);
        for i in 0..SIMD_TEST_SIZE { assert_eq!(a_simd[i].into_bigint(), expected[i].into_bigint()); }
    }

    #[test]
    fn simd_sub_assign_works() {
        let mut rng = ark_std::test_rng();
        let a: vec::Vec<SmallF16> = (0..SIMD_TEST_SIZE)
            .map(|_| SmallF16::rand(&mut rng))
            .collect();
        let b: vec::Vec<SmallF16> = (0..SIMD_TEST_SIZE)
            .map(|_| SmallF16::rand(&mut rng))
            .collect();

        let mut a_simd: vec::Vec<SmallF16Simd> = a.iter().map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap()).collect();
        let b_simd: vec::Vec<SmallF16Simd> = b.iter().map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap()).collect();

        let mut expected = a.clone();
        for (val_a, val_b) in expected.iter_mut().zip(b.iter()) {
            *val_a -= val_b;
        }

        SmallF16SimdConfig::sub_assign_simd(&mut a_simd, &b_simd);
        for i in 0..SIMD_TEST_SIZE {
            assert_eq!(a_simd[i].into_bigint(), expected[i].into_bigint());
        }
    }

    #[test]
    fn simd_div_assign_works() {
        let mut rng = ark_std::test_rng();
        let a: vec::Vec<SmallF16> = (0..SIMD_TEST_SIZE)
            .map(|_| SmallF16::rand(&mut rng))
            .collect();
        let b: vec::Vec<SmallF16> = (0..SIMD_TEST_SIZE)
            .map(|_| {
                let mut val = SmallF16::rand(&mut rng);
                // Ensure we don't divide by zero
                while val.is_zero() {
                    val = SmallF16::rand(&mut rng);
                }
                val
            })
            .collect();

        let mut a_simd: vec::Vec<SmallF16Simd> = a.iter().map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap()).collect();
        let b_simd: vec::Vec<SmallF16Simd> = b.iter().map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap()).collect();

        let mut expected = a.clone();
        for (val_a, val_b) in expected.iter_mut().zip(b.iter()) {
            *val_a /= val_b;
        }

        SmallF16SimdConfig::div_assign_simd(&mut a_simd, &b_simd);
        for i in 0..SIMD_TEST_SIZE {
            assert_eq!(a_simd[i].into_bigint(), expected[i].into_bigint());
        }
    }
    */
}

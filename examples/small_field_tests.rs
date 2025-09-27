// Example demonstrating comprehensive field arithmetic tests for SmallFp using test templates
// This shows how to use the test_small_field! macro similar to regular field tests

use ark_ff::ark_ff_macros::SmallFpConfig;
use ark_ff::{SmallFp, SmallFpConfig, BigInt, SqrtPrecomputation};
use ark_algebra_test_templates::*;

// M31 prime field: 2^31 - 1 = 2147483647
#[derive(SmallFpConfig)]
#[modulus = "2147483647"] // m31
#[generator = "7"]
#[backend = "standard"]
pub struct M31Config;

pub type M31SmallField = SmallFp<M31Config>;

// BabyBear prime field: 2^31 - 2^27 + 1 = 2013265921  
#[derive(SmallFpConfig)]
#[modulus = "2013265921"] // BabyBear
#[generator = "31"]
#[backend = "montgomery"]
pub struct BabyBearConfig;

pub type BabyBearSmallField = SmallFp<BabyBearConfig>;

// Small prime field for testing: 101
#[derive(SmallFpConfig)]
#[modulus = "101"]
#[generator = "2"]
#[backend = "standard"]
pub struct TinyFieldConfig;

pub type TinySmallField = SmallFp<TinyFieldConfig>;

// Create comprehensive test suites for each small field
test_small_field!(m31; M31SmallField; small_prime_field);
test_small_field!(babybear; BabyBearSmallField; small_prime_field);
test_small_field!(100; tiny; TinySmallField; small_prime_field);

fn main() {
    println!("Small field test templates example");
    println!("Run with: cargo test --example small_field_tests");
    
    // Demonstrate basic usage
    let a = M31SmallField::from(42u32);
    let b = M31SmallField::from(13u32);
    let c = a + b;
    println!("M31: {} + {} = {}", a, b, c);
    
    let a = BabyBearSmallField::from(42u32);
    let b = BabyBearSmallField::from(13u32);
    let c = a + b;
    println!("BabyBear: {} + {} = {}", a, b, c);
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use ark_ff::{Field, Zero, One};
    use ark_std::{UniformRand, rand::Rng};

    #[test]
    fn test_cross_field_consistency() {
        // Test that different implementations give consistent results for small values
        let mut rng = ark_std::test_rng();
        
        for _ in 0..100 {
            let val_u32 = rng.gen::<u16>() as u32; // Keep values small for cross-field testing
            if val_u32 < 101 { // Only test values valid in all fields
                let m31_val = M31SmallField::from(val_u32);
                let bb_val = BabyBearSmallField::from(val_u32);
                let tiny_val = TinySmallField::from(val_u32);
                
                // Test basic arithmetic consistency
                let m31_squared = m31_val * m31_val;
                let bb_squared = bb_val * bb_val;
                let tiny_squared = tiny_val * tiny_val;
                
                // All should compute the same result modulo their respective fields
                assert_eq!(m31_squared, M31SmallField::from((val_u32 * val_u32) % 2147483647));
                assert_eq!(bb_squared, BabyBearSmallField::from((val_u32 * val_u32) % 2013265921));
                assert_eq!(tiny_squared, TinySmallField::from((val_u32 * val_u32) % 101));
            }
        }
    }

    #[test]
    fn test_small_field_properties() {
        // Test specific properties of small fields
        
        // Test M31 (Mersenne prime)
        let m31_order = M31SmallField::from(2147483647u32 - 1);
        let m31_zero = M31SmallField::from(2147483647u32); // Should wrap to 0
        assert_eq!(m31_zero, M31SmallField::zero());
        
        // Test BabyBear properties
        let bb_generator = BabyBearSmallField::from(31u32);
        assert!(!bb_generator.is_zero());
        assert!(!bb_generator.is_one());
        
        // Test tiny field (all elements can be exhausted)
        let mut elements = std::collections::HashSet::new();
        for i in 0u32..101 {
            let elem = TinySmallField::from(i);
            elements.insert(elem);
        }
        assert_eq!(elements.len(), 101); // All elements should be distinct
    }
}
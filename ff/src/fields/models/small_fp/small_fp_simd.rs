use super::small_fp_backend::{SmallFp, SmallFpConfig};

pub trait SmallFpSimd: SmallFpConfig {
    /// Number of SIMD lanes supported is determined by the underlying primitive type size.
    const SIMD_LANES: usize;

    fn add_assign_simd(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]);
    fn mul_assign_simd(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]);
}

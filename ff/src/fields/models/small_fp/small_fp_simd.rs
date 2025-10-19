use super::small_fp_backend::{SmallFp, SmallFpConfig};

pub trait SmallFpSimd: SmallFpConfig {
    /// Number of SIMD lanes supported is determined by the underlying primitive type size.
    const SIMD_LANES: usize;

    fn simd_add_assign(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]);
    // fn simd_mul_assign(a: &mut [SmallFp<Self>], b: &[SmallFp<Self>]);
}

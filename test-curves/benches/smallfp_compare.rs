use ark_algebra_bench_templates::*;
use ark_ff::fields::{Fp128, MontBackend, SmallFp, SmallFpConfig};
use ark_test_curves::{fp128::Fq as Fp128Field, smallfp128::SmallFp128};

// Benchmark Fp128 (the standard MontBackend implementation)
f_bench!(prime, "Fp128", Fp128Field);

// Benchmark SmallFp128 (the SmallFp implementation we're optimizing)
f_bench!(prime, "SmallFp128", SmallFp128);

criterion_main!(
    fp128field::benches,
    smallfp128::benches,
);

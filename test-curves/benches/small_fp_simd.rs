use ark_ff::{PrimeField, SmallFpSimd, UniformRand};
use ark_std::vec::Vec;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ark_test_curves::smallfp16::{SmallF16, SmallF16Simd, SmallF16SimdConfig};

const BENCH_SIZE: usize = 1 << 20;

fn setup_bench() -> (Vec<SmallF16>, Vec<SmallF16>, Vec<SmallF16Simd>, Vec<SmallF16Simd>) {
    let mut rng = ark_std::test_rng();
    let a_scalar: Vec<SmallF16> = (0..BENCH_SIZE).map(|_| SmallF16::rand(&mut rng)).collect();
    let b_scalar: Vec<SmallF16> = (0..BENCH_SIZE).map(|_| SmallF16::rand(&mut rng)).collect();

    let a_simd: Vec<SmallF16Simd> = a_scalar
        .iter()
        .map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap())
        .collect();
    let b_simd: Vec<SmallF16Simd> = b_scalar
        .iter()
        .map(|x| SmallF16Simd::from_bigint(x.into_bigint()).unwrap())
        .collect();

    (a_scalar, b_scalar, a_simd, b_simd)
}

fn bench_add_assign(c: &mut Criterion) {
    let (a_scalar, b_scalar, a_simd, b_simd) = setup_bench();

    let mut group = c.benchmark_group("SmallFp AddAssign");

    group.bench_with_input(
        BenchmarkId::new("Standard", BENCH_SIZE),
        &(a_scalar, b_scalar),
        |bencher, (a, b_vec)| {
            bencher.iter_with_setup(
                || a.clone(),
                |mut a_clone| {
                    for (x, y) in a_clone.iter_mut().zip(b_vec.iter()) {
                        *x += y;
                    }
                },
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("SIMD", BENCH_SIZE),
        &(a_simd, b_simd),
        |bencher, (a, b_vec)| {
            bencher.iter_with_setup(
                || a.clone(),
                |mut a_clone| {
                    SmallF16SimdConfig::simd_add_assign(&mut a_clone, b_vec);
                },
            )
        },
    );

    group.finish();
}

fn bench_mul_assign(c: &mut Criterion) {
    let (a_scalar, b_scalar, a_simd, b_simd) = setup_bench();

    let mut group = c.benchmark_group("SmallFp MulAssign");

    group.bench_with_input(
        BenchmarkId::new("Standard", BENCH_SIZE),
        &(a_scalar, b_scalar),
        |bencher, (a, b_vec)| {
            bencher.iter_with_setup(
                || a.clone(),
                |mut a_clone| {
                    for (x, y) in a_clone.iter_mut().zip(b_vec.iter()) {
                        *x *= y;
                    }
                },
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("SIMD", BENCH_SIZE),
        &(a_simd, b_simd),
        |bencher, (a, b_vec)| {
            bencher.iter_with_setup(
                || a.clone(),
                |mut a_clone| {
                    SmallF16SimdConfig::simd_mul_assign(&mut a_clone, b_vec);
                },
            )
        },
    );

    group.finish();
}
criterion_group!(benches, bench_add_assign, bench_mul_assign);
criterion_main!(benches);
use ark_ff::{PrimeField, SmallFpSimd, UniformRand};
use ark_std::vec::Vec;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use ark_test_curves::smallfp8::{SmallF8, SmallF8Simd, SmallF8SimdConfig};
use ark_test_curves::smallfp16::{SmallF16, SmallF16Simd, SmallF16SimdConfig};
use ark_test_curves::smallfp32::{SmallF32, SmallF32Simd, SmallF32SimdConfig};

const BENCH_SIZE: usize = 1 << 12;

macro_rules! bench_field {
    ($field_name:ident, $field:ty, $field_simd:ty, $field_simd_config:ty) => {
        fn $field_name(c: &mut Criterion) {
            let mut rng = ark_std::test_rng();
            let a_scalar: Vec<$field> = (0..BENCH_SIZE).map(|_| <$field>::rand(&mut rng)).collect();
            let b_scalar: Vec<$field> = (0..BENCH_SIZE).map(|_| <$field>::rand(&mut rng)).collect();

            let a_simd: Vec<$field_simd> = a_scalar
                .iter()
                .map(|x| <$field_simd>::from_bigint(x.into_bigint()).unwrap())
                .collect();
            let b_simd: Vec<$field_simd> = b_scalar
                .iter()
                .map(|x| <$field_simd>::from_bigint(x.into_bigint()).unwrap())
                .collect();

            let mut group = c.benchmark_group(stringify!($field_name));

            group.bench_with_input(
                BenchmarkId::new("add_assign_standard", BENCH_SIZE),
                &(&a_scalar, &b_scalar),
                |bencher, (a, b_vec)| {
                    bencher.iter_with_setup(
                        || a.to_vec(),
                        |mut a_clone| {
                            for (x, y) in a_clone.iter_mut().zip(b_vec.iter()) {
                                *x += y;
                            }
                        },
                    )
                },
            );

            group.bench_with_input(
                BenchmarkId::new("add_assign_simd", BENCH_SIZE),
                &(&a_simd, &b_simd),
                |bencher, (a, b_vec)| {
                    bencher.iter_with_setup(
                        || a.to_vec(),
                        |mut a_clone| {
                            <$field_simd_config>::add_assign_simd(&mut a_clone, b_vec);
                        },
                    )
                },
            );

            group.bench_with_input(
                BenchmarkId::new("mul_assign_simd", BENCH_SIZE),
                &(&a_simd, &b_simd),
                |bencher, (a, b_vec)| {
                    bencher.iter_with_setup(
                        || a.to_vec(),
                        |mut a_clone| {
                            <$field_simd_config>::mul_assign_simd(&mut a_clone, b_vec);
                        },
                    )
                },
            );

            group.bench_with_input(
                BenchmarkId::new("sub_assign_standard", BENCH_SIZE),
                &(&a_scalar, &b_scalar),
                |bencher, (a, b_vec)| {
                    bencher.iter_with_setup(
                        || a.to_vec(),
                        |mut a_clone| {
                            for (x, y) in a_clone.iter_mut().zip(b_vec.iter()) {
                                *x -= y;
                            }
                        },
                    )
                },
            );

            group.bench_with_input(
                BenchmarkId::new("sub_assign_simd", BENCH_SIZE),
                &(&a_simd, &b_simd),
                |bencher, (a, b_vec)| {
                    bencher.iter_with_setup(
                        || a.to_vec(),
                        |mut a_clone| {
                            <$field_simd_config>::sub_assign_simd(&mut a_clone, b_vec);
                        },
                    )
                },
            );

            group.bench_with_input(
                BenchmarkId::new("div_assign_standard", BENCH_SIZE),
                &(&a_scalar, &b_scalar),
                |bencher, (a, b_vec)| {
                    bencher.iter_with_setup(
                        || a.to_vec(),
                        |mut a_clone| {
                            for (x, y) in a_clone.iter_mut().zip(b_vec.iter()) {
                                *x /= y;
                            }
                        },
                    )
                },
            );

            group.bench_with_input(
                BenchmarkId::new("div_assign_simd", BENCH_SIZE),
                &(&a_simd, &b_simd),
                |bencher, (a, b_vec)| {
                    bencher.iter_with_setup(
                        || a.to_vec(),
                        |mut a_clone| {
                            <$field_simd_config>::div_assign_simd(&mut a_clone, b_vec);
                        },
                    )
                },
            );

            group.finish();
        }
    };
}

bench_field!(bench_f8, SmallF8, SmallF8Simd, SmallF8SimdConfig);
bench_field!(bench_f16, SmallF16, SmallF16Simd, SmallF16SimdConfig);
bench_field!(bench_f32, SmallF32, SmallF32Simd, SmallF32SimdConfig);

criterion_group!(benches, bench_f8, bench_f16, bench_f32);
criterion_main!(benches);
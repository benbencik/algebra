use ark_algebra_bench_templates::*;
use ark_ff::fields::{Fp128, Fp64, MontBackend, MontConfig};
use ark_ff::{Field, UniformRand};
use ark_test_curves::{
    smallfp128::SmallF128Mont,
    smallfp64::SmallF64MontGoldilock,
    smallfp32::{SmallF32MontBabybear, SmallF32MontM31},
};
use criterion::BenchmarkGroup;

use ark_ff::ark_ff_macros::SmallFpConfig;
use ark_ff::{BigInt, SmallFp, SmallFpConfig, SqrtPrecomputation};

#[derive(MontConfig)]
#[modulus = "143244528689204659050391023439224324689"]
#[generator = "3"]
pub struct F128Config;
pub type F128 = Fp128<MontBackend<F128Config, 2>>;

#[derive(MontConfig)]
#[modulus = "18446744069414584321"]
#[generator = "7"]
pub struct F64Config;
pub type F64 = Fp64<MontBackend<F64Config, 1>>;

#[derive(MontConfig)]
#[modulus = "2013265921"]
#[generator = "3"]
pub struct F32ConfigBabybear;
pub type F32Babybear = Fp64<MontBackend<F32ConfigBabybear, 1>>;

#[derive(MontConfig)]
#[modulus = "2147483647"]
#[generator = "7"]
pub struct F32Config;
pub type F32 = Fp64<MontBackend<F32Config, 1>>;

#[derive(MontConfig)]
#[modulus = "65521"]
#[generator = "17"]
pub struct F16Config;
pub type F16 = Fp64<MontBackend<F16Config, 1>>;

#[derive(SmallFpConfig)]
#[modulus = "65521"]
#[generator = "17"]
#[backend = "montgomery"]
pub struct F16ConfigMont;
pub type SmallF16Mont = SmallFp<F16ConfigMont>;

#[derive(MontConfig)]
#[modulus = "251"]
#[generator = "6"]
pub struct F8Config;
pub type F8 = Fp64<MontBackend<F8Config, 1>>;

#[derive(SmallFpConfig)]
#[modulus = "251"]
#[generator = "6"]
#[backend = "montgomery"]
pub struct SmallF8ConfigMont;
pub type SmallF8Mont = SmallFp<SmallF8ConfigMont>;

fn bench_add<'a, F: Field + Copy, M: criterion::measurement::Measurement>(
    group: &mut BenchmarkGroup<'a, M>,
    name: &str,
    arr: &[F],
) {
    let samples = arr.len();
    group.bench_function(name, |b| {
        let mut i = 0;
        b.iter(|| {
            i = (i + 1) % samples;
            let mut tmp = arr[i];
            tmp += arr[(i*1) % samples];
            tmp
        })
    });
}

fn bench_mul<'a, F: Field + Copy, M: criterion::measurement::Measurement>(
    group: &mut BenchmarkGroup<'a, M>,
    name: &str,
    arr: &[F],
) {
    let samples = arr.len();
    group.bench_function(name, |b| {
        let mut i = 0;
        b.iter(|| {
            i = (i + 1) % samples;
            let mut tmp = arr[i];
            tmp *= arr[(i*1) % samples];
            tmp
        })
    });
}

fn bench_addition(c: &mut criterion::Criterion) {
    use ark_ff::PrimeField;
    const SAMPLES: usize = 10_000;
    let mut rng = ark_std::test_rng();
    let mut group = c.benchmark_group("Add");

    let smallf8 = (0..SAMPLES).map(|_| SmallF8Mont::rand(&mut rng)).collect::<Vec<_>>();
    let f8: Vec<_> = smallf8.iter().map(|x| F8::from(x.into_bigint().0[0])).collect();
    bench_add(&mut group, "00-F8", &f8);
    bench_add(&mut group, "00-SmallF8Mont", &smallf8);

    let smallf16 = (0..SAMPLES).map(|_| SmallF16Mont::rand(&mut rng)).collect::<Vec<_>>();
    let f16: Vec<_> = smallf16.iter().map(|x| F16::from(x.into_bigint().0[0])).collect();
    bench_add(&mut group, "01-F16", &f16);
    bench_add(&mut group, "01-SmallF16", &smallf16);

    let smallf32_m31 = (0..SAMPLES).map(|_| SmallF32MontM31::rand(&mut rng)).collect::<Vec<_>>();
    let f32_m31: Vec<_> = smallf32_m31.iter().map(|x| F32::from(x.into_bigint().0[0])).collect();
    bench_add(&mut group, "02-F32-M31", &f32_m31);
    bench_add(&mut group, "02-SmallF32-M31", &smallf32_m31);

    let smallf32_bb = (0..SAMPLES).map(|_| SmallF32MontBabybear::rand(&mut rng)).collect::<Vec<_>>();
    let f32_bb: Vec<_> = smallf32_bb.iter().map(|x| F32Babybear::from(x.into_bigint().0[0])).collect();
    bench_add(&mut group, "03-F32-Babybear", &f32_bb);
    bench_add(&mut group, "03-SmallF32-Babybear", &smallf32_bb);

    let smallf64 = (0..SAMPLES).map(|_| SmallF64MontGoldilock::rand(&mut rng)).collect::<Vec<_>>();
    let f64_: Vec<_> = smallf64.iter().map(|x| F64::from(x.into_bigint().0[0])).collect();
    bench_add(&mut group, "04-F64-Goldilock", &f64_);
    bench_add(&mut group, "04-SmallF64-Goldilock", &smallf64);

    let smallf128 = (0..SAMPLES).map(|_| SmallF128Mont::rand(&mut rng)).collect::<Vec<_>>();
    let f128: Vec<_> = smallf128
        .iter()
        .map(|x| {
            let bigint = x.into_bigint();
            F128::from(bigint)
        })
        .collect();
    bench_add(&mut group, "05-F128", &f128);
    bench_add(&mut group, "05-SmallF128", &smallf128);

    group.finish();
}

fn bench_multiplication(c: &mut criterion::Criterion) {
    use ark_ff::PrimeField;
    const SAMPLES: usize = 10_000;
    let mut rng = ark_std::test_rng();
    let mut group = c.benchmark_group("Mul");

    let smallf8 = (0..SAMPLES).map(|_| SmallF8Mont::rand(&mut rng)).collect::<Vec<_>>();
    let f8: Vec<_> = smallf8.iter().map(|x| F8::from(x.into_bigint().0[0])).collect();
    bench_mul(&mut group, "00-F8", &f8);
    bench_mul(&mut group, "00-SmallF8", &smallf8);

    let smallf16 = (0..SAMPLES).map(|_| SmallF16Mont::rand(&mut rng)).collect::<Vec<_>>();
    let f16: Vec<_> = smallf16.iter().map(|x| F16::from(x.into_bigint().0[0])).collect();
    bench_mul(&mut group, "01-F16", &f16);
    bench_mul(&mut group, "01-SmallF16", &smallf16);

    let smallf32_m31 = (0..SAMPLES).map(|_| SmallF32MontM31::rand(&mut rng)).collect::<Vec<_>>();
    let f32_m31: Vec<_> = smallf32_m31.iter().map(|x| F32::from(x.into_bigint().0[0])).collect();
    bench_mul(&mut group, "02-F32-M31", &f32_m31);
    bench_mul(&mut group, "02-SmallF32-M31", &smallf32_m31);

    let smallf32_bb = (0..SAMPLES).map(|_| SmallF32MontBabybear::rand(&mut rng)).collect::<Vec<_>>();
    let f32_bb: Vec<_> = smallf32_bb.iter().map(|x| F32Babybear::from(x.into_bigint().0[0])).collect();
    bench_mul(&mut group, "03-F32-Babybear", &f32_bb);
    bench_mul(&mut group, "03-SmallF32-Babybear", &smallf32_bb);

    let smallf64 = (0..SAMPLES).map(|_| SmallF64MontGoldilock::rand(&mut rng)).collect::<Vec<_>>();
    let f64_: Vec<_> = smallf64.iter().map(|x| F64::from(x.into_bigint().0[0])).collect();
    bench_mul(&mut group, "04-F64-Goldilock", &f64_);
    bench_mul(&mut group, "04-SmallF64-Goldilock", &smallf64);

    let smallf128 = (0..SAMPLES).map(|_| SmallF128Mont::rand(&mut rng)).collect::<Vec<_>>();
    let f128: Vec<_> = smallf128
        .iter()
        .map(|x| {
            let bigint = x.into_bigint();
            F128::from(bigint)
        })
        .collect();
    bench_mul(&mut group, "05-F128", &f128);
    bench_mul(&mut group, "05-SmallF128Mont", &smallf128);

    group.finish();
}

criterion_group!(addition_benches, bench_addition);
criterion_group!(multiplication_benches, bench_multiplication);

criterion_main!(
    addition_benches,
    multiplication_benches,
);

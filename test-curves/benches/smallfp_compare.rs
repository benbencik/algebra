use ark_algebra_bench_templates::*;
use ark_ff::{Fp128, Fp64, MontBackend, MontConfig, Field, UniformRand};
use ark_test_curves::{
    smallfp128::SmallF128Mont,
    smallfp64::SmallF64MontGoldilock,
    smallfp32::{SmallF32MontBabybear, SmallF32MontM31},
    smallfp16::SmallF16Mont,
    smallfp8::SmallF8MontM7,
};
use criterion::BenchmarkGroup;

#[derive(MontConfig)]
#[modulus = "143244528689204659050391023439224324689"]
#[generator = "3"]
pub struct F128Config;
pub type F128Generic = Fp128<MontBackend<F128Config, 2>>;

#[derive(MontConfig)]
#[modulus = "18446744069414584321"]
#[generator = "7"]
pub struct F64ConfigGoldilocks;
pub type F64Goldilocks = Fp64<MontBackend<F64ConfigGoldilocks, 1>>;

#[derive(MontConfig)]
#[modulus = "2013265921"]
#[generator = "31"]
pub struct F32ConfigBabybear;
pub type F32Babybear = Fp64<MontBackend<F32ConfigBabybear, 1>>;

#[derive(MontConfig)]
#[modulus = "2147483647"]
#[generator = "7"]
pub struct F32ConfigM31;
pub type F32M31 = Fp64<MontBackend<F32ConfigM31, 1>>;

#[derive(MontConfig)]
#[modulus = "65521"]
#[generator = "17"]
pub struct F16Config;
pub type F16Generic = Fp64<MontBackend<F16Config, 1>>;

#[derive(MontConfig)]
#[modulus = "8191"]
#[generator = "17"]
pub struct F16ConfigM13;
pub type F16M13 = Fp64<MontBackend<F16ConfigM13, 1>>;

#[derive(MontConfig)]
#[modulus = "127"]
#[generator = "3"]
pub struct F8ConfigM7;
pub type F8M7 = Fp64<MontBackend<F8ConfigM7, 1>>;

fn bench_add<'a, F: Field + Copy, M: criterion::measurement::Measurement>(
    group: &mut BenchmarkGroup<'a, M>,
    name: &str,
    arr: &[F],
    arr2: &[F],
) {
    let samples = arr.len();
    group.bench_function(name, |b| {
        let mut i = 0;
        b.iter(|| {
            i = (i + 1) % samples;
            let mut tmp = arr[i];
            tmp += arr2[i];
            tmp
        })
    });
}

fn bench_mul<'a, F: Field + Copy, M: criterion::measurement::Measurement>(
    group: &mut BenchmarkGroup<'a, M>,
    name: &str,
    arr: &mut [F],
    arr2: &[F],
) {
    let samples = arr.len();
    group.bench_function(name, |b| {
        let mut i = 0;
        b.iter(|| {
            i = (i + 1) % samples;
            let mut tmp = arr[i];
            tmp.mul_assign(&arr2[i]);
            tmp
        })
    });
}

fn bench_inv<'a, F: Field + Copy, M: criterion::measurement::Measurement>(
    group: &mut BenchmarkGroup<'a, M>,
    name: &str,
    arr: &[F],
    _arr2: &[F],
) {
    let samples = arr.len();
    group.bench_function(name, |b| {
        let mut i = 0;
        b.iter(|| {
            i = (i + 1) % samples;
            arr[i].inverse()
        })
    });
}


macro_rules! bench_operations {
    ($group:expr, $samples:expr, $rng:expr, $bench_fn:ident, $($name:expr, $small_ty:ty, $large_ty:ty),* $(,)?) => {
        $(
            let mut arr = (0..$samples).map(|_| <$small_ty>::rand(&mut $rng)).collect::<Vec<_>>();
            let arr_2 = (0..$samples).map(|_| <$small_ty>::rand(&mut $rng)).collect::<Vec<_>>();
            let mut fp_arr = (0..$samples).map(|_| <$large_ty>::rand(&mut $rng)).collect::<Vec<_>>();
            let fp_arr_2 = (0..$samples).map(|_| <$large_ty>::rand(&mut $rng)).collect::<Vec<_>>();
            $bench_fn(&mut $group, &format!("{}-{}", $name, stringify!($large_ty)), &mut fp_arr, &fp_arr_2);
            $bench_fn(&mut $group, &format!("{}-{}", $name, stringify!($small_ty)), &mut arr, &arr_2);
        )*
    };
}

fn bench_addition(c: &mut criterion::Criterion) {
    const SAMPLES: usize = 1_000_000;
    let mut rng = ark_std::test_rng();
    let mut group = c.benchmark_group("Add");

    bench_operations!(
        group,
        SAMPLES,
        rng,
        bench_add,
        "01", SmallF8MontM7, F8M7,
        "02", SmallF16Mont, F16Generic,
        "03", SmallF32MontM31, F32M31,
        "04", SmallF32MontBabybear, F32Babybear,
        "05", SmallF64MontGoldilock, F64Goldilocks,
        "06", SmallF128Mont, F128Generic,
    );

    group.finish();
}

fn bench_multiplication(c: &mut criterion::Criterion) {
    const SAMPLES: usize = 1_000_000;
    let mut rng = ark_std::test_rng();
    let mut group = c.benchmark_group("Mul");

    bench_operations!(
        group,
        SAMPLES,
        rng,
        bench_mul,
        "01", SmallF8MontM7, F8M7,
        "02", SmallF16Mont, F16Generic,
        "03", SmallF32MontM31, F32M31,
        "04", SmallF32MontBabybear, F32Babybear,
        "05", SmallF64MontGoldilock, F64Goldilocks,
        "06", SmallF128Mont, F128Generic,
    );

    group.finish();
}

fn bench_inverse(c: &mut criterion::Criterion) {
    const SAMPLES: usize = 1_000_000;
    let mut rng = ark_std::test_rng();
    let mut group = c.benchmark_group("Inv");

    bench_operations!(
        group,
        SAMPLES,
        rng,
        bench_inv,
        "01", SmallF8MontM7, F8M7,
        "02", SmallF16Mont, F16Generic,
        "03", SmallF32MontM31, F32M31,
        "04", SmallF32MontBabybear, F32Babybear,
        "05", SmallF64MontGoldilock, F64Goldilocks,
        "06", SmallF128Mont, F128Generic,
    );

    group.finish();
}

criterion_group!(addition_benches, bench_addition);
criterion_group!(multiplication_benches, bench_multiplication);
criterion_group!(inverse_benches, bench_inverse);

criterion_main!(
    // addition_benches,
    multiplication_benches,
    // inverse_benches,
);

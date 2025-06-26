use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dynamic_data_experiments::{byte_data::{Params,Data}, encoder::{RSEncoder, BLSEncoder}};
use dynamic_data_experiments::traits::{DataMatrix, Encoder};

fn bench_rs_encode(c: &mut Criterion) {
    // test parameters
    let k = 100;
    let p = 100;
    let n = k + p;
    let m = 200;
    let params = Params { k, n, m };

    // generate a random data matrix once
    let data = Data::new_random(params.clone());

    c.bench_function("RSEncoder::encode", |b| {
        b.iter(|| {
            // clone data for each iteration to avoid mutating the original
            let mut d = black_box(data.clone());
            RSEncoder::encode(&mut d).expect("encode failed");
        });
    });
}

fn bench_bls_encode(c: &mut Criterion) {
    // test parameters
    let k = 100;
    let p = 100;
    let n = k + p;
    let m = 200;
    let params = Params { k, n, m };

    // generate a random data matrix once
    let data = Data::new_random(params.clone());

    c.bench_function("BLSEncoder::encode", |b| {
        b.iter(|| {
            let mut d = black_box(data.clone());
            BLSEncoder::encode(&mut d).expect("encode failed");
        });
    });
}

criterion_group!(benches, bench_rs_encode, bench_bls_encode);
criterion_main!(benches);

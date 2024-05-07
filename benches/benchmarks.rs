#[macro_use]
extern crate criterion;
use criterion::{black_box, Criterion};
use geo_types::LineString;
use polyline::{decode_polyline, encode_coordinates};
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[allow(unused_must_use)]
fn bench_encode(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    // These coordinates cover London, approximately
    let between_lon = Uniform::from(-6.379880..1.768960);
    let between_lat = Uniform::from(49.871159..55.811741);
    let mut v: Vec<[f64; 2]> = vec![];
    (0..10_000).for_each(|_| v.push([between_lon.sample(&mut rng), between_lat.sample(&mut rng)]));
    let res: LineString<f64> = v.into();

    c.bench_function("encode 10_000 coordinates at precision 1e-5", |b| {
        b.iter(|| {
            black_box(encode_coordinates(res.coords().copied(), 5));
        })
    });

    c.bench_function("encode 10_000 coordinates at precision 1e-6", |b| {
        b.iter(|| {
            black_box(encode_coordinates(res.coords().copied(), 6));
        })
    });
}

#[allow(unused_must_use)]
fn bench_decode(c: &mut Criterion) {
    // comparable cpp (see e.g. Valhalla)
    let mut rng = StdRng::seed_from_u64(42);
    // These coordinates cover London, approximately
    let between_lon = Uniform::from(-6.379880..1.768960);
    let between_lat = Uniform::from(49.871159..55.811741);
    let mut v: Vec<[f64; 2]> = vec![];
    (0..10_000).for_each(|_| v.push([between_lon.sample(&mut rng), between_lat.sample(&mut rng)]));
    let res: LineString<f64> = v.into();
    let encoded = encode_coordinates(res, 6).unwrap();

    c.bench_function("decode 10_000 coordinates at precision 1e-5", |b| {
        b.iter(|| {
            black_box(decode_polyline(&encoded, 5));
        })
    });

    c.bench_function("decode 10_000 coordinates at precision 1e-6", |b| {
        b.iter(|| {
            black_box(decode_polyline(&encoded, 6));
        })
    });
}

criterion_group!(benches, bench_encode, bench_decode,);
criterion_main!(benches);

#[macro_use]
extern crate criterion;
use criterion::{black_box, Criterion};
use geo_types::Coord;
use polyline::{decode_polyline, encode_coordinates};
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn build_coords() -> Vec<Coord> {
    let mut rng = StdRng::seed_from_u64(42);
    // These coordinates cover London, approximately
    let between_lon = Uniform::from(-6.379880..1.768960);
    let between_lat = Uniform::from(49.871159..55.811741);
    (0..10_000)
        .map(|_| Coord {
            x: between_lon.sample(&mut rng),
            y: between_lat.sample(&mut rng),
        })
        .collect()
}

#[allow(unused_must_use)]
fn bench_encode(c: &mut Criterion) {
    let coords = build_coords();
    c.bench_function("encode 10_000 coordinates at precision 1e-5", |b| {
        b.iter(|| {
            black_box(encode_coordinates(coords.iter().copied(), 5).unwrap());
        })
    });

    c.bench_function("encode 10_000 coordinates at precision 1e-6", |b| {
        b.iter(|| {
            black_box(encode_coordinates(coords.iter().copied(), 6).unwrap());
        })
    });
}

#[allow(unused_must_use)]
fn bench_decode(c: &mut Criterion) {
    let coords = build_coords();

    c.bench_function("decode 10_000 coordinates at precision 1e-5", |b| {
        let encoded = encode_coordinates(coords.iter().copied(), 5).unwrap();
        b.iter(|| {
            black_box(decode_polyline(&encoded, 5).unwrap());
        })
    });

    c.bench_function("decode 10_000 coordinates at precision 1e-6", |b| {
        let encoded = encode_coordinates(coords.iter().copied(), 6).unwrap();
        b.iter(|| {
            black_box(decode_polyline(&encoded, 6).unwrap());
        })
    });
}

criterion_group!(benches, bench_encode, bench_decode,);
criterion_main!(benches);

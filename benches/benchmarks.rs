#[macro_use]
extern crate criterion;
use criterion::Criterion;
use geo_types::LineString;
use polyline::{decode_polyline, encode_coordinates};
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::thread_rng;

#[allow(unused_must_use)]
fn bench_encode(c: &mut Criterion) {
    let mut rng = thread_rng();
    // These coordinates cover London, approximately
    let between_lon = Uniform::from(-6.379880..1.768960);
    let between_lat = Uniform::from(49.871159..55.811741);
    let mut v: Vec<[f64; 2]> = vec![];
    (0..1000).for_each(|_| v.push([between_lon.sample(&mut rng), between_lat.sample(&mut rng)]));
    let res: LineString<f64> = v.into();
    c.bench_function("bench encode: 1000 coordinates", move |b| {
        b.iter(|| {
            encode_coordinates(res.clone(), 5);
        })
    });
}

#[allow(unused_must_use)]
fn bench_decode(c: &mut Criterion) {
    // comparable cpp (see e.g. Valhalla) decodes the same number of coords in around 500 µs
    let mut rng = thread_rng();
    // These coordinates cover London, approximately
    let between_lon = Uniform::from(-6.379880..1.768960);
    let between_lat = Uniform::from(49.871159..55.811741);
    let mut v: Vec<[f64; 2]> = vec![];
    (0..21501).for_each(|_| v.push([between_lon.sample(&mut rng), between_lat.sample(&mut rng)]));
    let res: LineString<f64> = v.into();
    let encoded = encode_coordinates(res.clone(), 6).unwrap();
    c.bench_function("bench decode: 21502 coordinates", move |b| {
        b.iter(|| {
            decode_polyline(&encoded, 6);
        })
    });
}

#[allow(unused_must_use)]
fn bench_polyline6_decoding(c: &mut Criterion) {
    c.bench_function("bench polyline6 decoding", move |b| {
        b.iter(|| {
            decode_polyline("_p~iF~ps|U_ulLnnqC_mqNvxq`@", 6).unwrap();
        })
    });
}

#[allow(unused_must_use)]
fn bench_polyline6_decoding_huge(c: &mut Criterion) {
    c.bench_function("bench HUGE polyline6 decoding", move |b| {
        b.iter(|| {
            decode_polyline(
                include_str!("../resources/route-geometry-sweden-west-coast.polyline6"),
                6,
            )
            .unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_encode,
    bench_decode,
    bench_polyline6_decoding,
    bench_polyline6_decoding_huge
);
criterion_main!(benches);

#[macro_use]
extern crate criterion;
use criterion::Criterion;
use geo_types::LineString;
use polyline::{decode_polyline, encode_coordinates};
use rand::distributions::{Distribution, Range};

#[allow(unused_must_use)]
fn bench_threads(c: &mut Criterion) {
    let num_coords = 10000;
    // These coordinates cover London, approximately
    let between_lon = Range::new(-6.379880, 1.768960);
    let between_lat = Range::new(49.871159, 55.811741);
    let mut rng = rand::thread_rng();
    let res = vec![[between_lat.sample(&mut rng), between_lon.sample(&mut rng)]; num_coords];
    let res: LineString<f64> = res.into();
    c.bench_function("bench threads", move |b| {
        b.iter(|| {
            encode_coordinates(res.clone(), 5);
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

criterion_group!(benches, bench_threads, bench_polyline6_decoding);
criterion_main!(benches);

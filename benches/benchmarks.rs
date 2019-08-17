#[macro_use]
extern crate criterion;
use criterion::Criterion;
use polyline::encode_coordinates;
use rand::distributions::{Distribution, Range};

#[allow(unused_must_use)]
fn bench_threads(c: &mut Criterion) {
    let num_coords = 10000;
    // These coordinates cover London, approximately
    let between_lon = Range::new(-6.379880, 1.768960);
    let between_lat = Range::new(49.871159, 55.811741);
    let mut rng = rand::thread_rng();
    let res = vec![[between_lat.sample(&mut rng), between_lon.sample(&mut rng)]; num_coords];
    c.bench_function("bench threads", move |b| b.iter(|| {
        encode_coordinates(&res, 5);
    }));
}

criterion_group!(benches, bench_threads);
criterion_main!(benches);

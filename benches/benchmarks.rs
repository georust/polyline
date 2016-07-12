#![feature(test)]
extern crate test;
use test::Bencher;
extern crate polyline;
use polyline::encode_coordinates;

extern crate rand;
use rand::distributions::{IndependentSample, Range};

#[bench]
#[allow(unused_must_use)]
fn bench_threads(b: &mut Bencher) {
    let num_coords = 10000;
    // These coordinates cover London, approximately
    let between_lon = Range::new(-6.379880, 1.768960);
    let between_lat = Range::new(49.871159, 55.811741);
    let mut rng = rand::thread_rng();
    let res = vec![[between_lat.ind_sample(&mut rng), between_lon.ind_sample(&mut rng)]; num_coords];
    b.iter(||{
        encode_coordinates(&res, 5);
    });
}

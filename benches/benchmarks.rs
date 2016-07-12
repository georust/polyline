#![feature(test)]
extern crate test;
use test::Bencher;
extern crate polyline;
use polyline::encode_coordinates;

#[bench]
#[allow(unused_must_use)]
fn bench_threads(b: &mut Bencher) {
    let res = vec![[38.5, -120.2], [40.7, -120.95], [430.252, -126.453]];
    b.iter(||{
        encode_coordinates(&res, 5);
    });
}

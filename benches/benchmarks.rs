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
    let res : LineString<f64> = res.into();
    c.bench_function("bench threads", move |b| b.iter(|| {
        encode_coordinates(res.clone(), 5);
    }));
}

fn bench_decode(c: &mut Criterion) {
    const poly: &str = r#"oxl~E}noyDBCJ?LED@PIZA\KR?h@UBBHEF?^GPIP?r@M`@Md@C`AWb@?XKvB]hAWfAMt@_@T?VG\IXK|@Sf@Un@G\ONMNGBMHIXMRE`@WFIRC\Wp@Mh@WHQBWJEHQJIBIG_A@{@QsC@eAC]?{@ImA?g@CcBHaBCeAEo@@aDCqAFSJKJGv@?RKd@IFM?UDPBAB@?BB?C@AHDEGC?G@?G@^Pr@Ln@DRDzCCvABf@U^_@HCn@?h@EfD_@pAKl@KfAG^Gb@AHHVZDHDPP`CDnAHf@b@dAj@t@FBBA~AiBvBs@NAPH`@d@b@\z@t@j@x@BB@A"#;

    c.bench_function("bench decode", move |b| {
        b.iter(|| {
            decode_polyline(&poly, 5);
        })
    });
}

criterion_group!(benches, bench_threads, bench_decode);
criterion_main!(benches);

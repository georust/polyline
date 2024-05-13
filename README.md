# polyline

[![polyline](https://avatars1.githubusercontent.com/u/10320338?v=4&s=50)](https://github.com/georust)

[![polyline on Crates.io](https://img.shields.io/crates/v/polyline.svg?color=brightgreen)](https://crates.io/crates/polyline)
[![Documentation](https://img.shields.io/docsrs/polyline/latest.svg)](https://docs.rs/polyline)
[![Discord](https://img.shields.io/discord/598002550221963289)](https://discord.gg/Fp2aape)

Fast Google Encoded Polyline encoding & decoding in Rust.

# Example
```rust
use polyline;
use geo_types::line_string;
let coord = line_string![(x: -120.2, y: 38.5), (x: -120.95, y: 40.7), (x: -126.453, y: 43.252)];
let output = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
let result = polyline::encode_coordinates(coord, 5).unwrap();
assert_eq!(result, output)
```

# A Note on Coordinate Order

This crate uses `Coord` and `LineString` types from the `geo-types` crate, which encodes coordinates in `(x, y)` / `(lon, lat)` order. The Polyline algorithm and its first-party documentation assumes the _opposite_ coordinate order. It is thus advisable to pay careful attention to the order of the coordinates you use for encoding and decoding.

[Documentation](https://docs.rs/polyline/)

# FFI
C-compatible FFI bindings for this crate are provided by the [polyline-ffi](https://crates.io/crates/polyline-ffi) crate.

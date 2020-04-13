# polyline

[![Crates.io](https://img.shields.io/crates/d/polyline.svg?maxAge=2592000?style=plastic)](https://crates.io/crates/polyline)
[![Build Status](https://travis-ci.org/georust/polyline.svg?branch=master)](https://travis-ci.org/georust/polyline)

Google Encoded Polyline encoding & decoding in Rust.

## A Note on Coordinate Order

This crate uses `Coordinate` and `LineString` types from the `geo-types` crate, which encodes coordinates in `(x, y)` order. The Polyline algorithm and first-party documentation assumes the _opposite_ coordinate order. It is thus advisable to pay careful attention to the order of the coordinates you use for encoding and decoding.

[Documentation](https://docs.rs/polyline/)

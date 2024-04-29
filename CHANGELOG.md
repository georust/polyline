# Changelog

## Unreleased

* inspired by HERE's [flexpolyline](https://github.com/heremaps/flexible-polyline):
  * provide alt. implementation of both encode() and decode() based on LUT
  * rework encoder delta-computation (performance improvement)
* correctly differentiate character and overflow errors when decoding

## 0.10.2

* fix decoder crashing with out-of-bounds error (https://github.com/georust/polyline/pull/37):
  * protect against invalid polylines
  * protect against potential overflow when shifting
  * performance hit: 10-12%

## 0.10.1

* Fix dependencies to officially drop geo-types 0.6 - it was already
  effectively dropped with the bump to 0.10.0 (it wouldn't compile), so this
  isn't an additionally breaking change.

## 0.10.0

* Update Coordinates to Coord due to geo-types change
  * This is a BREAKING change for geo-types 0.6 users
* Apply clippy suggestions
* Update dependencies
* Refactor decoding logic for perf improvement (https://github.com/georust/polyline/pull/28)

## 0.9.0
* Update `geo-types` dependency to allow for 0.6 or 0.7
* Switch CI to Github actions

## 0.8.0
* Bump geo-types dependency to 0.6

## 0.7.3
* Add note on coordinate order

## 0.7.1
* Bump `geo-types` to [0.5](https://github.com/georust/polyline/pull/21)

## 0.7.0

* [Relicense to MIT / Apache v2](https://github.com/georust/polyline/pull/18)

## 0.6.0

* [Mark `rand` as a dev-dependency.](https://github.com/georust/polyline/pull/12)
* [Switch to criterion to enable stable benchmarks](https://github.com/georust/polyline/pull/15)

## 0.5.0

* [`decode_polyline()` now accepts a string slice instead of a `String`](https://github.com/georust/polyline/pull/10).

## 0.4.0

* Now accepts either a slice or a vec as argument to encode_polyline.

## 0.3.0

* [Now tests for invalid coordinates in decoded strings](https://github.com/georust/polyline/pull/4)

## 0.2.0

* [Basic error handling for en– and decoding](https://github.com/tmcw/polyline/pull/3): rust-polyline
  now returns a `Result` object, allowing it to handle incorrectly
  encoded polylines.

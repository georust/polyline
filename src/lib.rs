//! # Google Encoded Polyline encoding & decoding in Rust
//!
//! [Polyline](https://developers.google.com/maps/documentation/utilities/polylinealgorithm)
//! is a lossy compression algorithm that allows you to store a series of coordinates as a
//! single string.
//!
//! # Example
//!
//! ```
//! use polyline;
//! use geo_types::line_string;
//!
//! let coord = line_string![(x: -120.2, y: 38.5), (x: -120.95, y: 40.7), (x: -126.453, y: 43.252)];
//! let output = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
//! let result = polyline::encode_coordinates(coord, 5).unwrap();
//! assert_eq!(result, output)
//! ```
//!
//!# A Note on Coordinate Order
//!
//! This crate uses `Coordinate` and `LineString` types from the `geo-types` crate, which encodes coordinates
//! in `(x, y)` order. The Polyline algorithm and first-party documentation assumes the _opposite_ coordinate order.
//! It is thus advisable to pay careful attention to the order of the coordinates you use for encoding and decoding.

use geo_types::{Coord, LineString};
use std::{char, cmp};

const MIN_LONGITUDE: f64 = -180.0;
const MAX_LONGITUDE: f64 = 180.0;
const MIN_LATITUDE: f64 = -90.0;
const MAX_LATITUDE: f64 = 90.0;

fn scale(n: f64, factor: i32) -> i64 {
    let scaled = n * (f64::from(factor));
    scaled.round() as i64
}

// Bounds checking for input values
fn check<T>(to_check: T, bounds: (T, T)) -> Result<T, T>
where
    T: cmp::PartialOrd + Copy,
{
    match to_check {
        to_check if bounds.0 <= to_check && to_check <= bounds.1 => Ok(to_check),
        _ => Err(to_check),
    }
}

fn encode(delta: i64, encoded: &mut String) {
    let mut value = delta << 1;
    if value < 0 {
        value = !value;
    }
    _encode(value as u64, encoded);
}

fn _encode(mut value: u64, result: &mut String) {
    const ENCODING_TABLE: &str =
        "?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

    while value >= 0x20 {
        let pos = (value & 0x1F) | 0x20;
        let from_char = ENCODING_TABLE.as_bytes()[pos as usize] as char;
        result.push(from_char);
        value >>= 5;
    }
    let from_char = ENCODING_TABLE.as_bytes()[value as usize] as char;
    result.push(from_char);
}

/// Encodes a Google Encoded Polyline.
///
/// # Examples
///
/// ```
/// use polyline;
/// use geo_types::line_string;
///
/// let coords = line_string![(x: 2.0, y: 1.0), (x: 4.0, y: 3.0)];
/// let encoded_vec = polyline::encode_coordinates(coords, 5).unwrap();
/// ```
pub fn encode_coordinates<C>(coordinates: C, precision: u32) -> Result<String, String>
where
    C: IntoIterator<Item = Coord<f64>>,
{
    let base: i32 = 10;
    let factor: i32 = base.pow(precision);
    let mut encoded: String = "".to_string();

    let mut current: Coord<i64> = Coord { x: 0, y: 0 };
    let mut previous: Coord<i64> = Coord { x: 0, y: 0 };

    for (i, a) in coordinates.into_iter().enumerate() {
        current.x = scale(a.x, factor);
        current.y = scale(a.y, factor);
        check(a.y, (MIN_LATITUDE, MAX_LATITUDE))
            .map_err(|e| format!("Latitude error at position {0}: {1}", i, e))?;
        check(a.x, (MIN_LONGITUDE, MAX_LONGITUDE))
            .map_err(|e| format!("Longitude error at position {0}: {1}", i, e))?;
        encode(current.y - previous.y, &mut encoded);
        encode(current.x - previous.x, &mut encoded);
        previous = current;
    }
    Ok(encoded)
}

/// Decodes a Google Encoded Polyline.
///
/// # Examples
///
/// ```
/// use polyline;
///
/// let decodedPolyline = polyline::decode_polyline(&"_p~iF~ps|U_ulLnnqC_mqNvxq`@", 5);
/// ```
pub fn decode_polyline(polyline: &str, precision: u32) -> Result<LineString<f64>, String> {
    let mut index = 0;
    let mut lat: i64 = 0;
    let mut lng: i64 = 0;
    let mut coordinates = vec![];
    let base: i32 = 10;
    let factor = i64::from(base.pow(precision));

    let chars = polyline.as_bytes();

    while index < chars.len() {
        let (latitude_change, new_index) = trans(chars, index)?;
        if new_index >= chars.len() {
            break;
        }
        let (longitude_change, new_index) = trans(chars, new_index)?;
        index = new_index;

        lat += latitude_change;
        lng += longitude_change;

        coordinates.push([lng as f64 / factor as f64, lat as f64 / factor as f64]);
    }

    Ok(coordinates.into())
}

fn trans(chars: &[u8], mut index: usize) -> Result<(i64, usize), String> {
    #[rustfmt::skip]
    const DECODING_TABLE: &[i8] = &[
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1,  0,  1,  2,  3,  4,  5,  6,
         7,  8,  9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        27, 28, 29, 30, 31, 32, 33, 34, 35, 36,
        37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
        47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
        57, 58, 59, 60, 61, 62, 63, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1,
    ];

    let mut shift = 0;
    let mut result = 0;
    let mut byte;
    loop {
        byte = DECODING_TABLE[chars[index] as usize];
        if byte < 1 {
            return Err(format!("Cannot decode character at index {}", index));
        }
        result |= (byte as u64 & 0x1f) << shift;
        index += 1;
        shift += 5;
        if byte < 0x20 {
            break;
        }
    }

    let coordinate_change = if (result & 1) > 0 {
        !(result >> 1)
    } else {
        result >> 1
    } as i64;
    Ok((coordinate_change, index))
}

#[cfg(test)]
mod tests {

    use super::decode_polyline;
    use super::encode_coordinates;
    use geo_types::LineString;

    struct TestCase {
        input: LineString<f64>,
        output: &'static str,
    }

    #[test]
    fn precision5() {
        let test_cases = vec![
            TestCase {
                input: vec![[2.0, 1.0], [4.0, 3.0]].into(),
                output: "_ibE_seK_seK_seK",
            },
            TestCase {
                input: vec![[-120.2, 38.5], [-120.95, 40.7], [-126.453, 43.252]].into(),
                output: "_p~iF~ps|U_ulLnnqC_mqNvxq`@",
            },
        ];
        for test_case in test_cases {
            assert_eq!(
                encode_coordinates(test_case.input.clone(), 5).unwrap(),
                test_case.output
            );
            assert_eq!(
                decode_polyline(test_case.output, 5).unwrap(),
                test_case.input
            );
        }
    }

    #[test]
    fn precision6() {
        let test_cases = vec![
            TestCase {
                input: vec![[2.0, 1.0], [4.0, 3.0]].into(),
                output: "_c`|@_gayB_gayB_gayB",
            },
            TestCase {
                input: vec![[-120.2, 38.5], [-120.95, 40.7], [-126.453, 43.252]].into(),
                output: "_izlhA~rlgdF_{geC~ywl@_kwzCn`{nI",
            },
        ];
        for test_case in test_cases {
            assert_eq!(
                encode_coordinates(test_case.input.clone(), 6).unwrap(),
                test_case.output
            );
            assert_eq!(
                decode_polyline(test_case.output, 6).unwrap(),
                test_case.input
            );
        }
    }

    #[test]
    // coordinates close to each other (below precision) should work
    fn rounding_error() {
        let poly = "cr_iI}co{@?dB";
        let res: LineString<f64> = vec![[9.9131118, 54.0702648], [9.9126013, 54.0702578]].into();
        assert_eq!(encode_coordinates(res, 5).unwrap(), poly);
        assert_eq!(
            decode_polyline(poly, 5).unwrap(),
            vec![[9.91311, 54.07026], [9.91260, 54.07026]].into()
        );
    }

    #[test]
    // emoji is decodable but messes up data
    // TODO: handle this case in the future?
    fn broken_string() {
        let s = "_p~iF~ps|U_u🗑lLnnqC_mqNvxq`@";
        let expected: Result<_, _> = Err("Cannot decode character at index 12".to_string());
        assert_eq!(decode_polyline(s, 5), expected);
    }

    #[test]
    fn invalid_string() {
        let s = "invalid_polyline_that_should_be_handled_gracefully";
        let expected: Result<_, _> = Err("Cannot decode character at index 12".to_string());
        assert_eq!(decode_polyline(s, 5), expected);
    }

    #[test]
    fn another_invalid_string() {
        let s = "ugh_ugh";
        let expected: Result<_, _> = Err("Cannot decode character at index 12".to_string());
        assert_eq!(decode_polyline(s, 5), expected);
    }

    #[test]
    #[should_panic]
    // Can't have a latitude > 90.0
    fn bad_coords() {
        let res: LineString<f64> =
            vec![[-120.2, 38.5], [-120.95, 40.7], [-126.453, 430.252]].into();
        encode_coordinates(res, 5).unwrap();
    }

    #[test]
    fn should_not_trigger_overflow() {
        decode_polyline(
            include_str!("../resources/route-geometry-sweden-west-coast.polyline6"),
            6,
        )
        .unwrap();
    }
}

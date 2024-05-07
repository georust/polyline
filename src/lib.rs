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

fn encode(current: f64, previous: f64, factor: i32, output: &mut String) -> Result<(), String> {
    let current = scale(current, factor);
    let previous = scale(previous, factor);
    let mut coordinate = (current - previous) << 1;
    if (current - previous) < 0 {
        coordinate = !coordinate;
    }
    while coordinate >= 0x20 {
        let from_char = char::from_u32(((0x20 | (coordinate & 0x1f)) + 63) as u32)
            .ok_or("Couldn't convert character")?;
        output.push(from_char);
        coordinate >>= 5;
    }
    let from_char = char::from_u32((coordinate + 63) as u32).ok_or("Couldn't convert character")?;
    output.push(from_char);
    Ok(())
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

    let mut output = String::new();
    let mut b = Coord { x: 0.0, y: 0.0 };

    for (i, a) in coordinates.into_iter().enumerate() {
        check(a.y, (MIN_LATITUDE, MAX_LATITUDE))
            .map_err(|e| format!("Latitude error at position {0}: {1}", i, e))?;
        check(a.x, (MIN_LONGITUDE, MAX_LONGITUDE))
            .map_err(|e| format!("Longitude error at position {0}: {1}", i, e))?;
        encode(a.y, b.y, factor, &mut output)?;
        encode(a.x, b.x, factor, &mut output)?;
        b = a;
    }
    Ok(output)
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
    let mut shift = 0;
    let mut result = 0;
    let mut byte;
    loop {
        byte = chars[index] as u64;
        if byte < 63 || (shift > 64 - 5) {
            return Err(format!("Cannot decode character at index {}", index));
        }
        byte -= 63;
        result |= (byte & 0x1f) << shift;
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
        let res = vec![
            [-120.2, 38.5],
            [-120.95, 2306360.53104],
            [-126.453, 2306363.08304],
        ]
        .into();
        assert_eq!(decode_polyline(s, 5).unwrap(), res);
    }

    #[test]
    #[should_panic]
    fn invalid_string() {
        let s = "invalid_polyline_that_should_be_handled_gracefully";
        decode_polyline(s, 6).unwrap();
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

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

use std::{char, cmp};
use geo_types::{Coordinate, LineString};

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

fn encode(current: f64, previous: f64, factor: i32) -> Result<String, String> {
    let current = scale(current, factor);
    let previous = scale(previous, factor);
    let mut coordinate = (current - previous) << 1;
    if (current - previous) < 0 {
        coordinate = !coordinate;
    }
    let mut output: String = "".to_string();
    while coordinate >= 0x20 {
        let from_char = char::from_u32(((0x20 | (coordinate & 0x1f)) + 63) as u32)
            .ok_or("Couldn't convert character")?;
        output.push(from_char);
        coordinate >>= 5;
    }
    let from_char = char::from_u32((coordinate + 63) as u32).ok_or("Couldn't convert character")?;
    output.push(from_char);
    Ok(output)
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
    C: IntoIterator<Item=Coordinate<f64>>,
{
    let base: i32 = 10;
    let factor: i32 = base.pow(precision);

    let mut output = "".to_string();
    let mut b = Coordinate { x: 0.0, y: 0.0 };

    for (i, a) in coordinates.into_iter().enumerate() {
        check(a.y, (MIN_LATITUDE, MAX_LATITUDE))
            .map_err(|e| format!("Latitude error at position {0}: {1}", i, e).to_string())?;
        check(a.x, (MIN_LONGITUDE, MAX_LONGITUDE))
            .map_err(|e| format!("Longitude error at position {0}: {1}", i, e).to_string())?;
        output = output + &encode(a.y, b.y, factor)?;
        output = output + &encode(a.x, b.x, factor)?;
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
    let chars = polyline.as_bytes();
    let mut index = 0;
    let mut lat: i64 = 0;
    let mut lng: i64 = 0;
    let mut coordinates = vec![];
    let base: i32 = 10;
    let factor = i64::from(base.pow(precision)) as f64;

    coordinates.reserve(chars.len());

    while index < polyline.len() {
        let mut shift = 0;
        let mut result = 0;
        let mut byte;

        loop {
            byte = chars.get(index).ok_or("Couldn't decode Polyline")? - 63;
            result |= (byte as u64 & 0x1f) << shift;
            index += 1;
            shift += 5;
            if byte < 0x20 {
                break;
            }
        }

        let latitude_change: i64 = if (result & 1) > 0 {
            !(result >> 1)
        } else {
            result >> 1
        } as i64;

        shift = 0;
        result = 0;

        loop {
            byte = chars.get(index).ok_or("Couldn't decode Polyline")? - 63;
            index += 1;
            result |= (byte as u64 & 0x1f) << shift;
            shift += 5;
            if byte < 0x20 {
                break;
            }
        }

        let longitude_change: i64 = if (result & 1) > 0 {
            !(result >> 1)
        } else {
            result >> 1
        } as i64;

        lat += latitude_change;
        lng += longitude_change;

        coordinates.push(Coordinate {
            y: lat as f64 / factor,
            x: lng as f64 / factor,
        });
    }

    Ok(LineString::new(coordinates))
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
    fn it_works() {
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
                decode_polyline(&test_case.output, 5).unwrap(),
                test_case.input
            );
        }
    }

    #[test]
    // coordinates close to each other (below precision) should work
    fn rounding_error() {
        let poly = "cr_iI}co{@?dB";
        let res : LineString<f64> = vec![[9.9131118, 54.0702648], [9.9126013, 54.0702578]].into();
        assert_eq!(
            encode_coordinates(res, 5).unwrap(),
            poly
        );
        assert_eq!(
            decode_polyline(&poly, 5).unwrap(),
            vec![[9.91311, 54.07026], [9.91260, 54.07026]].into()
        );
    }

    #[test]
    #[should_panic]
    // emoji can't be decoded
    fn broken_string() {
        let s = "_p~iF~ps|U_u🗑lLnnqC_mqNvxq`@";
        let res = vec![[-120.2, 38.5], [-120.95, 40.7], [-126.453, 43.252]].into();
        assert_eq!(decode_polyline(&s, 5).unwrap(), res);
    }

    #[test]
    #[should_panic]
    // Can't have a latitude > 90.0
    fn bad_coords() {
        let s = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
        let res : LineString<f64> = vec![[-120.2, 38.5], [-120.95, 40.7], [-126.453, 430.252]].into();
        assert_eq!(encode_coordinates(res, 5).unwrap(), s);
    }
}

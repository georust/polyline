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

pub mod errors;
use errors::PolylineError;

use geo_types::{Coord, LineString};
use std::char;
use std::iter::{Enumerate, Peekable};

const MIN_LONGITUDE: f64 = -180.0;
const MAX_LONGITUDE: f64 = 180.0;
const MIN_LATITUDE: f64 = -90.0;
const MAX_LATITUDE: f64 = 90.0;

fn scale(n: f64, factor: i32) -> i64 {
    let scaled = n * (f64::from(factor));
    scaled.round() as i64
}

#[inline(always)]
fn encode(delta: i64, output: &mut String) -> Result<(), PolylineError> {
    let mut value = delta << 1;
    if value < 0 {
        value = !value;
    }
    while value >= 0x20 {
        let from_char = char::from_u32(((0x20 | (value & 0x1f)) + 63) as u32)
            .ok_or(PolylineError::DecodeCharError)?;
        output.push(from_char);
        value >>= 5;
    }
    let from_char = char::from_u32((value + 63) as u32).ok_or(PolylineError::DecodeCharError)?;
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
pub fn encode_coordinates<C>(coordinates: C, precision: u32) -> Result<String, PolylineError>
where
    C: IntoIterator<Item = Coord<f64>>,
{
    let base: i32 = 10;
    let factor: i32 = base.pow(precision);

    let mut output = String::new();
    let mut previous = Coord { x: 0, y: 0 };

    for (i, next) in coordinates.into_iter().enumerate() {
        if !(MIN_LATITUDE..=MAX_LATITUDE).contains(&next.y) {
            return Err(PolylineError::LatitudeCoordError {
                coord: next.y,
                idx: i,
            });
        }
        if !(MIN_LONGITUDE..=MAX_LONGITUDE).contains(&next.x) {
            return Err(PolylineError::LongitudeCoordError {
                coord: next.x,
                idx: i,
            });
        }

        let scaled_next = Coord {
            x: scale(next.x, factor),
            y: scale(next.y, factor),
        };
        encode(scaled_next.y - previous.y, &mut output)?;
        encode(scaled_next.x - previous.x, &mut output)?;
        previous = scaled_next;
    }
    Ok(output)
}

/// Decodes a Google Encoded Polyline.
///
/// Returns an error if the polyline is invalid or if the decoded coordinates are out of bounds.
///
/// # Examples
///
/// ```
/// use polyline;
///
/// let decoded_polyline = polyline::decode_polyline(&"_p~iF~ps|U_ulLnnqC_mqNvxq`@", 5);
/// ```
pub fn decode_polyline(polyline: &str, precision: u32) -> Result<LineString<f64>, PolylineError> {
    let mut scaled_lat: i64 = 0;
    let mut scaled_lon: i64 = 0;
    let mut coordinates = vec![];
    let base: i32 = 10;
    let factor = i64::from(base.pow(precision));

    let mut chars = polyline.as_bytes().iter().copied().enumerate().peekable();

    while let Some((lat_start, _)) = chars.peek().copied() {
        let latitude_change = decode_next(&mut chars)?;
        scaled_lat += latitude_change;
        let lat = scaled_lat as f64 / factor as f64;
        if !(MIN_LATITUDE..=MAX_LATITUDE).contains(&lat) {
            return Err(PolylineError::LatitudeCoordError {
                coord: lat,
                idx: lat_start,
            });
        }

        let Some((lon_start, _)) = chars.peek().copied() else {
            return Err(PolylineError::NoLongError { idx: lat_start });
        };
        let longitude_change = decode_next(&mut chars)?;
        scaled_lon += longitude_change;
        let lon = scaled_lon as f64 / factor as f64;
        if !(MIN_LONGITUDE..=MAX_LONGITUDE).contains(&lon) {
            return Err(PolylineError::LongitudeCoordError {
                coord: lon,
                idx: lon_start,
            });
        }

        coordinates.push([lon, lat]);
    }

    Ok(coordinates.into())
}

fn decode_next(
    chars: &mut Peekable<Enumerate<impl std::iter::Iterator<Item = u8>>>,
) -> Result<i64, PolylineError> {
    let mut shift = 0;
    let mut result = 0;
    for (idx, mut byte) in chars.by_ref() {
        if byte < 63 || (shift > 64 - 5) {
            return Err(PolylineError::DecodeError { idx });
        }
        byte -= 63;
        result |= ((byte & 0x1f) as u64) << shift;
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
    Ok(coordinate_change)
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
    fn broken_string() {
        let s = "_p~iF~ps|U_u🗑lLnnqC_mqNvxq`@";
        let err = decode_polyline(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::LatitudeCoordError { coord, idx } => {
                assert_eq!(coord, 2306360.53104);
                assert_eq!(idx, 10);
            }
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn invalid_string() {
        let s = "invalid_polyline_that_should_be_handled_gracefully";
        let err = decode_polyline(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::DecodeError { idx } => assert_eq!(idx, 12),
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn another_invalid_string() {
        let s = "ugh_ugh";
        let err = decode_polyline(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::LatitudeCoordError { coord, idx } => {
                assert_eq!(coord, 49775.95019);
                assert_eq!(idx, 0);
            }
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn bad_coords() {
        // Can't have a latitude > 90.0
        let res: LineString<f64> =
            vec![[-120.2, 38.5], [-120.95, 40.7], [-126.453, 430.252]].into();
        let err = encode_coordinates(res, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::LatitudeCoordError { coord, idx } => {
                assert_eq!(coord, 430.252);
                assert_eq!(idx, 2);
            }
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn should_not_trigger_overflow() {
        decode_polyline(
            include_str!("../resources/route-geometry-sweden-west-coast.polyline6"),
            6,
        )
        .unwrap();
    }

    #[test]
    fn limits() {
        let res: LineString<f64> = vec![[-180.0, -90.0], [180.0, 90.0], [0.0, 0.0]].into();
        let polyline = "~fdtjD~niivI_oiivI__tsmT~fdtjD~niivI";
        assert_eq!(
            encode_coordinates(res.coords().copied(), 6).unwrap(),
            polyline
        );
        assert_eq!(decode_polyline(polyline, 6).unwrap(), res);
    }

    #[test]
    fn truncated() {
        let input = LineString::from(vec![[2.0, 1.0], [4.0, 3.0]]);
        let polyline = "_ibE_seK_seK_seK";
        assert_eq!(
            encode_coordinates(input.coords().copied(), 5).unwrap(),
            polyline
        );
        assert_eq!(decode_polyline(polyline, 5).unwrap(), input);

        let truncated_polyline = "_ibE_seK_seK";
        let err = decode_polyline(truncated_polyline, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::NoLongError { idx } => {
                assert_eq!(idx, 8);
            }
            _ => panic!("Got wrong error"),
        }
    }
}

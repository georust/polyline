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

use geo_types::{Coord, CoordFloat, LineString};
use std::char;
use std::iter::{Enumerate, Peekable};

const MIN_LONGITUDE: f64 = -180.0;
const MAX_LONGITUDE: f64 = 180.0;
const MIN_LATITUDE: f64 = -90.0;
const MAX_LATITUDE: f64 = 90.0;

fn scale<T: CoordFloat>(n: T, factor: T) -> Result<i64, PolylineError<T>> {
    let scaled = n * factor;
    scaled.round().to_i64().ok_or(PolylineError::NumericCastFailure)
}

#[inline(always)]
fn encode<T: CoordFloat>(delta: i64, output: &mut String) -> Result<(), PolylineError<T>> {
    let mut value = delta << 1;
    if value < 0 {
        value = !value;
    }
    while value >= 0x20 {
        let from_char = char::from_u32(((0x20 | (value & 0x1f)) + 63) as u32)
            .ok_or(PolylineError::EncodeToCharError)?;
        output.push(from_char);
        value >>= 5;
    }
    let from_char = char::from_u32((value + 63) as u32).ok_or(PolylineError::EncodeToCharError)?;
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
pub fn encode_coordinates<C, T: CoordFloat>(coordinates: C, precision: u32) -> Result<String, PolylineError<T>>
where
    C: IntoIterator<Item = Coord<T>>,
{
    let base: i32 = 10;
    let Some(factor) = T::from(base.pow(precision)) else {
        return Err(PolylineError::NumericCastFailure)
    };

    let mut output = String::new();
    let mut previous = Coord { x: 0, y: 0 };

    for (i, next) in coordinates.into_iter().enumerate() {
        if !(T::from(MIN_LATITUDE)..=T::from(MAX_LATITUDE)).contains(&next.y.into()) {
            return Err(PolylineError::LatitudeCoordError {
                coord: next.y,
                idx: i,
            });
        }
        if !(T::from(MIN_LONGITUDE)..=T::from(MAX_LONGITUDE)).contains(&next.x.into()) {
            return Err(PolylineError::LongitudeCoordError {
                coord: next.x,
                idx: i,
            });
        }

        let scaled_next = Coord {
            x: scale(next.x, factor)?,
            y: scale(next.y, factor)?,
        };
        encode(scaled_next.y - previous.y, &mut output).map_err(|_: PolylineError<T>| {
            PolylineError::CoordEncodingError {
                coord: next,
                idx: i,
            }
        })?;
        encode(scaled_next.x - previous.x, &mut output).map_err(|_: PolylineError<T>| {
            PolylineError::CoordEncodingError {
                coord: next,
                idx: i,
            }
        })?;
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
/// let decoded_polyline = polyline::decode_polyline::<f64>(&"_p~iF~ps|U_ulLnnqC_mqNvxq`@", 5);
/// ```
pub fn decode_polyline<T: CoordFloat>(polyline: &str, precision: u32) -> Result<LineString<T>, PolylineError<T>> {
    let mut scaled_lat: i64 = 0;
    let mut scaled_lon: i64 = 0;
    let mut coordinates = vec![];
    let base: i32 = 10;
    let Some(factor) = T::from(base.pow(precision)) else {
        return Err(PolylineError::NumericCastFailure)
    };

    let mut chars = polyline.as_bytes().iter().copied().enumerate().peekable();

    while let Some((lat_start, _)) = chars.peek().copied() {
        let latitude_change = decode_next(&mut chars)?;
        scaled_lat += latitude_change;
        let lat = T::from(scaled_lat).ok_or(PolylineError::NumericCastFailure)? / factor;
        if !(MIN_LATITUDE..=MAX_LATITUDE).contains(&lat.to_f64().ok_or(PolylineError::NumericCastFailure)?) {
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
        let lon = T::from(scaled_lon).ok_or(PolylineError::NumericCastFailure)? / factor;
        if !(MIN_LONGITUDE..=MAX_LONGITUDE).contains(&lon.to_f64().ok_or(PolylineError::NumericCastFailure)?) {
            return Err(PolylineError::LongitudeCoordError {
                coord: lon,
                idx: lon_start,
            });
        }

        coordinates.push(Coord { x: lon, y: lat });
    }

    Ok(LineString::new(coordinates))
}

fn decode_next<T: CoordFloat>(
    chars: &mut Peekable<Enumerate<impl Iterator<Item = u8>>>,
) -> Result<i64, PolylineError<T>> {
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
    fn broken_string_f64() {
        let s = "_p~iF~ps|U_u🗑lLnnqC_mqNvxq`@";
        let err = decode_polyline::<f64>(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::LatitudeCoordError { coord, idx } => {
                assert_eq!(coord, 2306360.53104);
                assert_eq!(idx, 10);
            }
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn broken_string_f32() {
        let s = "_p~iF~ps|U_u🗑lLnnqC_mqNvxq`@";
        let err = decode_polyline::<f32>(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::LatitudeCoordError { coord, idx } => {
                assert_eq!(coord, 2306360.53104);
                assert_eq!(idx, 10);
            }
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn invalid_string_f64() {
        let s = "invalid_polyline_that_should_be_handled_gracefully";
        let err = decode_polyline::<f64>(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::DecodeError { idx } => assert_eq!(idx, 12),
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn invalid_string_f32() {
        let s = "invalid_polyline_that_should_be_handled_gracefully";
        let err = decode_polyline::<f32>(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::DecodeError { idx } => assert_eq!(idx, 12),
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn another_invalid_string_f64() {
        let s = "ugh_ugh";
        let err = decode_polyline::<f64>(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::LatitudeCoordError { coord, idx } => {
                assert_eq!(coord, 49775.95019);
                assert_eq!(idx, 0);
            }
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn another_invalid_string_f32() {
        let s = "ugh_ugh";
        let err = decode_polyline::<f32>(s, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::LatitudeCoordError { coord, idx } => {
                assert_eq!(coord, 49775.95019);
                assert_eq!(idx, 0);
            }
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn bad_coords_f64() {
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
    fn bad_coords_f32() {
        // Can't have a latitude > 90.0
        let res: LineString<f32> =
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
    fn should_not_trigger_overflow_f64() {
        decode_polyline::<f64>(
            include_str!("../resources/route-geometry-sweden-west-coast.polyline6"),
            6,
        )
        .unwrap();
    }

    #[test]
    fn should_not_trigger_overflow_f32() {
        decode_polyline::<f32>(
            include_str!("../resources/route-geometry-sweden-west-coast.polyline6"),
            6,
        )
        .unwrap();
    }

    #[test]
    fn limits_f64() {
        let res: LineString<f64> = vec![[-180.0, -90.0], [180.0, 90.0], [0.0, 0.0]].into();
        let polyline = "~fdtjD~niivI_oiivI__tsmT~fdtjD~niivI";
        assert_eq!(
            encode_coordinates(res.coords().copied(), 6).unwrap(),
            polyline
        );
        assert_eq!(decode_polyline(polyline, 6).unwrap(), res);
    }

    #[test]
    fn limits_f32() {
        let res: LineString<f32> = vec![[-180.0, -90.0], [180.0, 90.0], [0.0, 0.0]].into();
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
        let err = decode_polyline::<f64>(truncated_polyline, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::NoLongError { idx } => {
                assert_eq!(idx, 8);
            }
            _ => panic!("Got wrong error"),
        }
    }

    #[test]
    fn truncated_f32() {
        let input = LineString::from(vec![[2.0f32, 1.0f32], [4.0f32, 3.0f32]]);
        let polyline = "_ibE_seK_seK_seK";
        assert_eq!(
            encode_coordinates(input.coords().copied(), 5).unwrap(),
            polyline
        );
        assert_eq!(decode_polyline(polyline, 5).unwrap(), input);

        let truncated_polyline = "_ibE_seK_seK";
        let err = decode_polyline::<f32>(truncated_polyline, 5).unwrap_err();
        match err {
            crate::errors::PolylineError::NoLongError { idx } => {
                assert_eq!(idx, 8);
            }
            _ => panic!("Got wrong error"),
        }
    }
}

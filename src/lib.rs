//! # Google Encoded Polyline encoding & decoding in Rust
//!
//! [Polyline](https://developers.google.com/maps/documentation/utilities/polylinealgorithm)
//! is a lossy compression algorithm that allows you to store a series of coordinates as a
//! single string.
//!
//! The encoding process converts a binary value into a series of character codes for ASCII
//! characters using the familiar base64 encoding scheme: to ensure proper display of these
//! characters, encoded values are summed with 63 (the ASCII character '?') before converting
//! them into ASCII. The algorithm also checks for additional character codes for a given
//! point by checking the least significant bit of each byte group; if this bit is set to 1,
//! the point is not yet fully formed and additional data must follow.
//!
//! # Example
//!
//! Points: (38.5, -120.2), (40.7, -120.95), (43.252, -126.453)
//!
//! Encoded polyline: "_p~iF~ps|U_ulLnnqC_mqNvxq`@""
//! ```
//! use polyline;
//!
//! let coord = vec![[38.5, -120.2], [40.7, -120.95], [43.252, -126.453]];
//! let output = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
//! let result = polyline::encode_coordinates(&coord, 5).unwrap();
//! assert_eq!(result, output)
//! ```
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

fn encode(current: f64, previous: f64, factor: i32) -> Result<String, String> {
    let mut coordinate = (scale(current, factor) - scale(previous, factor)) << 1;
    if (current - previous) < 0.0 {
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

/// Encodes a Google Encoded Polyline. Accepts a slice,
/// or anything (such as a Vec) that can deref to a slice.
///
/// # Examples
///
/// ```
/// use polyline;
///
/// let coords_vec = vec![[1.0, 2.0], [3.0, 4.0]];
/// let encoded_vec = polyline::encode_coordinates(&coords_vec, 5).unwrap();
///
/// let coords_slice = [[1.0, 2.0], [3.0, 4.0]];
/// let encoded_slice = polyline::encode_coordinates(&coords_slice, 5).unwrap();
/// ```
pub fn encode_coordinates(coordinates: &[[f64; 2]], precision: u32) -> Result<String, String> {
    if coordinates.is_empty() {
        return Ok("".to_string());
    }
    for (i, pair) in coordinates.iter().enumerate() {
        check(pair[0], (MIN_LATITUDE, MAX_LATITUDE))
            .map_err(|e| format!("Latitude error at position {0}: {1}", i, e).to_string())?;

        check(pair[1], (MIN_LONGITUDE, MAX_LONGITUDE))
            .map_err(|e| format!("Longitude error at position {0}: {1}", i, e).to_string())?;
    }
    let base: i32 = 10;
    let factor: i32 = base.pow(precision);

    let mut output =
        encode(coordinates[0][0], 0.0, factor)? + &encode(coordinates[0][1], 0.0, factor)?;

    for (i, _) in coordinates.iter().enumerate().skip(1) {
        let a = coordinates[i];
        let b = coordinates[i - 1];
        output = output + &encode(a[0], b[0], factor)?;
        output = output + &encode(a[1], b[1], factor)?;
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
pub fn decode_polyline(polyline: &str, precision: u32) -> Result<Vec<[f64; 2]>, String> {
    let mut index = 0;
    let mut at_index;
    let mut lat: i64 = 0;
    let mut lng: i64 = 0;
    let mut coordinates = vec![];
    let base: i32 = 10;
    let factor = i64::from(base.pow(precision));

    while index < polyline.len() {
        let mut shift = 0;
        let mut result = 0;
        let mut byte;

        loop {
            at_index = polyline
                .chars()
                .nth(index)
                .ok_or("Couldn't decode Polyline")?;
            byte = at_index as u64 - 63;
            index += 1;
            result |= (byte & 0x1f) << shift;
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
            at_index = polyline
                .chars()
                .nth(index)
                .ok_or("Couldn't decode Polyline")?;
            byte = at_index as u64 - 63;
            index += 1;
            result |= (byte & 0x1f) << shift;
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

        coordinates.push([lat as f64 / factor as f64, lng as f64 / factor as f64]);
    }

    Ok(coordinates)
}

#[cfg(test)]
mod tests {

    use super::decode_polyline;
    use super::encode_coordinates;

    struct TestCase {
        input: Vec<[f64; 2]>,
        output: &'static str,
    }

    #[test]
    fn it_works() {
        let test_cases = vec![
            TestCase {
                input: vec![[1.0, 2.0], [3.0, 4.0]],
                output: "_ibE_seK_seK_seK",
            },
            TestCase {
                input: vec![[38.5, -120.2], [40.7, -120.95], [43.252, -126.453]],
                output: "_p~iF~ps|U_ulLnnqC_mqNvxq`@",
            },
        ];
        for test_case in test_cases {
            assert_eq!(
                encode_coordinates(&test_case.input, 5).unwrap(),
                test_case.output
            );
            assert_eq!(
                decode_polyline(&test_case.output, 5).unwrap(),
                test_case.input
            );
        }
    }

    #[test]
    #[should_panic]
    // emoji can't be decoded
    fn broken_string() {
        let s = "_p~iF~ps|U_u🗑lLnnqC_mqNvxq`@";
        let res = vec![[38.5, -120.2], [40.7, -120.95], [43.252, -126.453]];
        assert_eq!(decode_polyline(&s, 5).unwrap(), res);
    }

    #[test]
    #[should_panic]
    // Can't have a latitude > 90.0
    fn bad_coords() {
        let s = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
        let res = vec![[38.5, -120.2], [40.7, -120.95], [430.252, -126.453]];
        assert_eq!(encode_coordinates(&res, 5).unwrap(), s);
    }
}

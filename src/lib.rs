use std::{char, cmp};

mod ffi;
pub use ffi::Array;
pub use ffi::encode_coordinates_ffi;
pub use ffi::decode_polyline_ffi;
pub use ffi::drop_float_array;
pub use ffi::drop_cstring;

const MIN_LONGITUDE: f64 = -180.0;
const MAX_LONGITUDE: f64 = 180.0;
const MIN_LATITUDE: f64 = -90.0;
const MAX_LATITUDE: f64 = 90.0;

fn scale(n: f64, factor: i32) -> i64 {
    let scaled: f64 = n * (factor as f64);
    scaled.round() as i64
}

// Bounds checking for input values
fn check<T>(to_check: T, bounds: (T, T)) -> Result<T, T>
    where T: cmp::PartialOrd + Copy
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
        let from_char = try!(char::from_u32(((0x20 | (coordinate & 0x1f)) + 63) as u32)
            .ok_or("Couldn't convert character"));
        output.push(from_char);
        coordinate >>= 5;
    }
    let from_char = try!(char::from_u32((coordinate + 63) as u32)
        .ok_or("Couldn't convert character"));
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
        try!(check(pair[0], (MIN_LATITUDE, MAX_LATITUDE))
            .map_err(|e| format!("Latitude error at position {0}: {1}", i, e).to_string()));
        try!(check(pair[1], (MIN_LONGITUDE, MAX_LONGITUDE))
            .map_err(|e| format!("Longitude error at position {0}: {1}", i, e).to_string()));
    }
    let base: i32 = 10;
    let factor: i32 = base.pow(precision);

    let mut output = try!(encode(coordinates[0][0], 0.0, factor)) +
                     &try!(encode(coordinates[0][1], 0.0, factor));

    for (i, _) in coordinates.iter().enumerate().skip(1) {
        let a = coordinates[i];
        let b = coordinates[i - 1];
        output = output + &try!(encode(a[0], b[0], factor));
        output = output + &try!(encode(a[1], b[1], factor));
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
/// let decodedPolyline = polyline::decode_polyline("_p~iF~ps|U_ulLnnqC_mqNvxq`@".to_string(), 5);
/// ```
pub fn decode_polyline(str: String, precision: u32) -> Result<Vec<[f64; 2]>, String> {
    let mut index = 0;
    let mut lat: i64 = 0;
    let mut lng: i64 = 0;
    let mut coordinates = vec![];
    let base: i32 = 10;
    let factor: i64 = base.pow(precision) as i64;

    while index < str.len() {

        let mut shift = 0;
        let mut result = 0;
        let mut byte;

        while {
            let at_index = try!(str.chars().nth(index).ok_or("Couldn't decode Polyline"));
            byte = at_index as u64 - 63;
            index += 1;
            result |= (byte & 0x1f) << shift;
            shift += 5;
            byte >= 0x20
        } {
        }

        let latitude_change: i64 = if (result & 1) > 0 {
            !(result >> 1)
        } else {
            result >> 1
        } as i64;

        shift = 0;
        result = 0;

        while {
            let at_index = try!(str.chars().nth(index).ok_or("Couldn't decode Polyline"));
            byte = at_index as u64 - 63;
            index += 1;
            result |= (byte & 0x1f) << shift;
            shift += 5;
            byte >= 0x20
        } {
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

    use super::encode_coordinates;
    use super::decode_polyline;

    struct TestCase {
        input: Vec<[f64; 2]>,
        output: &'static str,
    }

    #[test]
    fn it_works() {
        let test_cases = vec![TestCase {
                                  input: vec![[1.0, 2.0], [3.0, 4.0]],
                                  output: "_ibE_seK_seK_seK",
                              },
                              TestCase {
                                  input: vec![[38.5, -120.2], [40.7, -120.95], [43.252, -126.453]],
                                  output: "_p~iF~ps|U_ulLnnqC_mqNvxq`@",
                              }];
        for test_case in test_cases {
            assert_eq!(encode_coordinates(&test_case.input, 5).unwrap(),
                       test_case.output);
            assert_eq!(decode_polyline(test_case.output.to_string(), 5).unwrap(),
                       test_case.input);
        }
    }

    #[test]
    #[should_panic]
    // emoji can't be decoded
    fn broken_string() {
        let s = "_p~iF~ps|U_u🗑lLnnqC_mqNvxq`@";
        let res = vec![[38.5, -120.2], [40.7, -120.95], [43.252, -126.453]];
        assert_eq!(decode_polyline(s.to_string(), 5).unwrap(), res);
    }

    #[test]
    #[should_panic]
    // Can't have a latitude > 90.0
    fn bad_coords() {
        let s = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
        let res = vec![[38.5, -120.2], [40.7, -120.95], [430.252, -126.453]];
        assert_eq!(encode_coordinates(&res, 5).unwrap(), s.to_string());
    }
}

//! This module exposes functions for accessing the Polyline encoding and decoding functions via FFI

use std::mem;
use std::slice;
use std::f64;
use std::ffi::{CStr, CString};

extern crate libc;
use self::libc::{c_char, c_void, uint32_t, size_t};

use super::encode_coordinates;
use super::decode_polyline;

#[repr(C)]
pub struct Array {
    pub data: *const c_void,
    pub len: size_t,
}


// Build an Array from &[[f64; 2]], so it can be leaked across the FFI boundary
impl<'a, T> From<&'a [[T; 2]]> for Array {
    fn from(sl: &'a [[T; 2]]) -> Self {
        let array = Array {
            data: sl.as_ptr() as *const c_void,
            len: sl.len() as size_t,
        };
        mem::forget(sl);
        array
    }
}

// Build &[[f64; 2]] from an Array, so it can be dropped
impl <'a>From<Array> for &'a [[f64; 2]] {
    fn from(arr: Array) -> Self {
        unsafe { slice::from_raw_parts(arr.data as *const [f64; 2], arr.len) }
    }
}

// Decode a Polyline into an Array
fn arr_from_string(incoming: String, precision: uint32_t) -> Array {
    let result: Array = match decode_polyline(incoming, precision) {
        Ok(res) => res.as_slice().into(),
        // should be easy to check for
        Err(_) => vec![[f64::NAN, f64::NAN]].as_slice().into(),
    };
    result.into()
}

// Decode an Array into a Polyline
fn string_from_arr(incoming: Array, precision: uint32_t) -> String {
    let result: String = match encode_coordinates(incoming.into(), precision) {
        Ok(res) => res,
        // we don't need to adapt the error
        Err(res) => res,
    };
    result
}

/// Convert a Polyline into an array of coordinates
///
/// Callers must pass two arguments:
///
/// - a pointer to `NUL`-terminated characters (`char*`)  
/// - an unsigned 32-bit `int` for precision (5 for Google Polylines, 6 for 
/// OSRM and Valhalla Polylines)
///
/// # Examples
///
/// ```
/// use std::ffi:CString
/// let input = CString::new("_ibE_seK_seK_seK").unwrap().as_ptr();
/// let result: Array = decode_polyline_ffi(input, 5);
/// let slice = unsafe { slice::from_raw_parts(result.data as *const [f64; 2], result.len) };
/// assert_eq!(slice, [[1.0, 2.0], [3.0, 4.0]]);
/// drop_float_array(result);
/// ```
///
/// A decoding failure will always return an array: `[[NaN, NaN]]`
///
/// Implementations calling this function **must** call [`drop_float_array`](fn.drop_float_array.html)
/// with the returned [Array](struct.Array.html), in order to free the memory it allocates.
///
/// # Safety
///
/// This function is unsafe because it accesses a raw pointer which could contain arbitrary data
#[no_mangle]
pub extern "C" fn decode_polyline_ffi(pl: *const c_char, precision: uint32_t) -> Array {
    let s: String = unsafe { CStr::from_ptr(pl).to_string_lossy().into_owned() };
    arr_from_string(s, precision)
}

/// Convert an array of coordinates into a Polyline  
///
/// Callers must pass two arguments:
///
/// - a [Struct](struct.Array.html) with two fields:  
///     - `data`, a void pointer to an array of floating-point lat, lon coordinates: `[[1.0, 2.0]]`  
///     - `len`, the length of the array being passed. Its type must be `size_t`: `1`
/// - an unsigned 32-bit `int` for precision (5 for Google Polylines, 6 for 
/// OSRM and Valhalla Polylines)
///
/// # Examples
///
/// ```
/// extern crate libc;
/// use libc::{c_void, size_t};
/// use std::ffi::CStr;
/// let input = vec![[1.0, 2.0], [3.0, 4.0]].as_slice();
/// let array = Array { data: sl.as_ptr() as *const c_void, len: sl.len() as size_t };
/// let output = "_ibE_seK_seK_seK".to_string(); 
/// let pl = encode_coordinates_ffi(array, 5);
/// let result = unsafe { CStr::from_ptr(pl).to_str().unwrap() };
/// assert_eq!(result, output);
/// drop_cstring(pl);
/// ```
///
/// A decoding failure will always return a string: "Couldn't decode Polyline"
/// 
/// Implementations calling this function **must** call [`drop_cstring`](fn.drop_cstring.html)
/// with the returned `c_char` pointer, in order to free the memory it allocates.
///
/// # Safety
///
/// This function is unsafe because it accesses a raw pointer which could contain arbitrary data
#[no_mangle]
pub extern "C" fn encode_coordinates_ffi(coords: Array, precision: uint32_t) -> *mut c_char {
    let s: String = string_from_arr(coords, precision);
    let result = match CString::new(s) {
        Ok(res) => res.into_raw(),
        // It's arguably better to fail noisily, but this is robust
        Err(_) => CString::new("Couldn't decode Polyline".to_string()).unwrap().into_raw(),
    };
    result
}

/// Free Array memory which Rust has allocated across the FFI boundary
///
/// # Safety
///
/// This function is unsafe because it accesses a raw pointer which could contain arbitrary data
#[no_mangle]
pub extern "C" fn drop_float_array(arr: Array) {
    if arr.data.is_null() {
        return;
    }
    let _: &[[f64; 2]] = arr.into();
}

/// Free CString memory which Rust has allocated across the FFI boundary
///
/// # Safety
///
/// This function is unsafe because it accesses a raw pointer which could contain arbitrary data
#[no_mangle]
pub extern "C" fn drop_cstring(p: *mut c_char) {
    let _ = unsafe { CString::from_raw(p) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use std::ffi::{CString, CStr};

    #[test]
    fn test_array_conversion() {
        let original = vec![[1.0, 2.0], [3.0, 4.0]];
        // move into an Array, and leak it
        let arr: Array = original.as_slice().into();
        // move back into a Vec -- leaked value still needs to be dropped
        let converted: &[[f64; 2]] = arr.into();
        assert_eq!(converted[0], [1.0, 2.0]);
        // drop it
        drop_float_array(converted.into());
    }

    #[test]
    fn test_drop_empty_float_array() {
        let original = vec![[1.0, 2.0], [3.0, 4.0]];
        // move into an Array, and leak it
        let mut arr: Array = original.as_slice().into();
        // zero Array contents
        arr.data = ptr::null();
        drop_float_array(arr);
    }

    #[test]
    fn test_coordinate_conversion() {
        let input = vec![[1.0, 2.0], [3.0, 4.0]];
        let output = "_ibE_seK_seK_seK";
        let input_arr: Array = input.as_slice().into();
        let transformed: String = super::string_from_arr(input_arr, 5);
        assert_eq!(transformed, output);
    }

    #[test]
    fn test_string_conversion() {
        let input = "_ibE_seK_seK_seK".to_string();
        let output = [[1.0, 2.0], [3.0, 4.0]];
        // String to Array
        let transformed: Array = super::arr_from_string(input, 5);
        // Array to Vec
        let transformed_arr: &[[f64; 2]] = transformed.into();
        assert_eq!(transformed_arr, output);
    }

    #[test]
    #[should_panic]
    fn test_bad_string_conversion() {
        let input = "_p~iF~ps|U_u🗑lLnnqC_mqNvxq`@".to_string();
        let output = vec![[1.0, 2.0], [3.0, 4.0]];
        // String to Array
        let transformed: Array = super::arr_from_string(input, 5);
        // Array to Vec
        let transformed_arr: &[[f64; 2]] = transformed.into();
        // this will fail, bc transformed_arr is [[NaN, NaN]]
        assert_eq!(transformed_arr, output.as_slice());
    }

    #[test]
    fn test_ffi_polyline_decoding() {
        let input = CString::new("_ibE_seK_seK_seK").unwrap().as_ptr();
        let result: &[[f64; 2]] = decode_polyline_ffi(input, 5).into();
        assert_eq!(result, [[1.0, 2.0], [3.0, 4.0]]);
        drop_float_array(result.into());
    }

    #[test]
    fn test_ffi_coordinate_encoding() {
        let input: Array = vec![[1.0, 2.0], [3.0, 4.0]].as_slice().into();
        let output = "_ibE_seK_seK_seK".to_string();
        let pl = encode_coordinates_ffi(input, 5);
        // Allocate a new String
        let result = unsafe { CStr::from_ptr(pl).to_str().unwrap() };
        assert_eq!(result, output);
        // Drop received FFI data
        drop_cstring(pl);
    }
}

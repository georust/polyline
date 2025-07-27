//! Errors that can occur during encoding / decoding of Polylines

use std::any::type_name;
use geo_types::{Coord, CoordFloat};

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum PolylineError<T: CoordFloat> {
    LongitudeCoordError {
        /// The coordinate value that caused the error due to being outside the range `-180.0..180.0`
        coord: T,
        /// The string index of the coordinate error
        idx: usize,
    },
    LatitudeCoordError {
        /// The coordinate value that caused the error due to being outside the range `-90.0..90.0`
        coord: T,
        /// The string index of the coordinate error
        idx: usize,
    },
    NoLongError {
        /// The string index of the missing longitude
        idx: usize,
    },
    DecodeError {
        /// The string index of the character that caused the decoding error
        idx: usize,
    },
    EncodeToCharError,
    CoordEncodingError {
        coord: Coord<T>,
        /// The array index of the coordinate error
        idx: usize,
    },
    /// Unable to convert a value to the desired type
    // TODO: Decide what info we want to express here
    NumericCastFailure
}

impl<T: CoordFloat> std::error::Error for PolylineError<T> {}
impl<T: CoordFloat> std::fmt::Display for PolylineError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PolylineError::LongitudeCoordError { coord, idx } => {
                write!(f, "longitude out of bounds: {:?} at position {}", coord, idx)
            }
            PolylineError::LatitudeCoordError { coord, idx } => {
                write!(f, "latitude out of bounds: {:?} at position {}", coord, idx)
            }
            PolylineError::DecodeError { idx } => {
                write!(f, "cannot decode character at index {}", idx)
            }
            PolylineError::NoLongError { idx } => {
                write!(f, "no longitude to go with latitude at index: {}", idx)
            }
            PolylineError::EncodeToCharError => write!(f, "couldn't encode character"),
            PolylineError::CoordEncodingError { coord, idx } => {
                write!(
                    f,
                    "the coordinate {:?} at index: {} could not be encoded",
                    coord, idx
                )
            },
            PolylineError::NumericCastFailure => {
                write!(f, "number is not representable as type {}", type_name::<T>())
            }
        }
    }
}

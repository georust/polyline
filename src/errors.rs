//! Errors that can occur during encoding / decoding of Polylines

use geo_types::Coord;

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum PolylineError {
    LongitudeCoordError {
        /// The coordinate value that caused the error due to being outside the range `-180.0..180.0`
        coord: f64,
        /// The string index of the coordinate error
        idx: usize,
    },
    LatitudeCoordError {
        /// The coordinate value that caused the error due to being outside the range `-90.0..90.0`
        coord: f64,
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
        coord: Coord<f64>,
        /// The array index of the coordinate error
        idx: usize,
    },
}

impl std::error::Error for PolylineError {}
impl std::fmt::Display for PolylineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PolylineError::LongitudeCoordError { coord, idx } => {
                write!(f, "longitude out of bounds: {} at position {}", coord, idx)
            }
            PolylineError::LatitudeCoordError { coord, idx } => {
                write!(f, "latitude out of bounds: {} at position {}", coord, idx)
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
            }
        }
    }
}

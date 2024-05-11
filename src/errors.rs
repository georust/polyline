//! Errors that can occur during encoding / decoding of Polylines

#[derive(Debug, PartialEq)]
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
    DecodeCharError,
}

impl std::error::Error for PolylineError {}
impl std::fmt::Display for PolylineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PolylineError::LongitudeCoordError { coord, idx } => {
                write!(f, "Invalid longitude: {} at position {}", coord, idx)
            }
            PolylineError::LatitudeCoordError { coord, idx } => {
                write!(f, "Invalid latitude: {} at position {}", coord, idx)
            }
            PolylineError::DecodeError { idx } => {
                write!(f, "Cannot decode character at index {}", idx)
            }
            PolylineError::NoLongError { idx } => {
                write!(f, "No longitude to go with latitude at index: {}", idx)
            }
            PolylineError::DecodeCharError => write!(f, "Couldn't decode character"),
        }
    }
}

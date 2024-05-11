use std::fmt::Write;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum PolylineError {
    LongitudeCoordError { coord: f64, idx: usize },
    LatitudeCoordError { coord: f64, idx: usize },
    NoLongError { idx: usize },
    DecodeError { idx: usize },
    DecodeCharError,
}

impl std::error::Error for PolylineError {}
impl std::fmt::Display for PolylineError {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::new();
        match self {
            PolylineError::LongitudeCoordError { coord, idx } => {
                write!(&mut s, "Invalid longitude: {} at position {}", coord, idx)
            }
            PolylineError::LatitudeCoordError { coord, idx } => {
                write!(&mut s, "Invalid latitude: {} at position {}", coord, idx)
            }
            PolylineError::DecodeError { idx } => {
                write!(&mut s, "Cannot decode character at index {}", idx)
            }
            PolylineError::NoLongError { idx } => {
                write!(&mut s, "No longitude to go with latitude at index: {}", idx)
            }
            PolylineError::DecodeCharError => write!(&mut s, "Couldn't decode character"),
        }
    }
}

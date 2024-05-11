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

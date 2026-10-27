use crate::services::validation_service::path_segment::PathSegment;
use crate::services::validation_service::request_segment::RequestSegment;

/// A trait to identify if a key is a parameter (e.g., in URL routing).
pub trait ParametrizedMatcher {
    /// Returns true if the key is a parameter.
    fn is_parameter(&self) -> bool;
}

impl ParametrizedMatcher for PathSegment {
    fn is_parameter(&self) -> bool {
        matches!(self, PathSegment::Parameter)
    }
}

impl ParametrizedMatcher for RequestSegment {
    fn is_parameter(&self) -> bool {
        matches!(self, RequestSegment::Path(PathSegment::Parameter))
    }
}

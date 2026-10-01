use super::OriginContext;

/// Adds request context to final audit events while preserving the response or error.
pub trait EnrichFromOrigin<T> {
    fn enrich_from_origin(value: T, context: OriginContext) -> T;
}

use actix_web::http::header::HeaderValue;

/// Contract for tokens that can be parsed from an HTTP header and expose a stable token id.
///
/// Implementors are expected to validate header format in `TryFrom<HeaderValue>` and provide
/// an identifier suitable for audit and trace records.
pub trait TokenWithId: TryFrom<HeaderValue, Error = anyhow::Error> {
    /// Returns the token identifier used in audit events.
    fn id(&self) -> String;
}

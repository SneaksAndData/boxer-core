use serde::Deserialize;

/// Represents the token validation settings used by boxer validator code
#[derive(Debug, Deserialize)]
pub struct TokenValidationSettings {
    pub audience: String,
    pub issuer: String,

    /// We use JSON-encoded string for signature settings since the validator must support multiple
    /// signatures for seamless key rotation. Unfortunately, the config-rs crate does not support
    /// deserializing directly into a Vec or HashMap from environment variables.
    pub keys: String,
}

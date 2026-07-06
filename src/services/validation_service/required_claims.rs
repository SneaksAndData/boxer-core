use cedar_policy::Entity;

/// Represents a container of JWT claims required for the token validation.
pub trait RequiredClaims {
    /// Returns the validator schema id from the claims.
    fn get_validator_schema_id(&self) -> String;

    /// Returns the parsed principal from the claims.
    fn get_principal(&self) -> Entity;
}

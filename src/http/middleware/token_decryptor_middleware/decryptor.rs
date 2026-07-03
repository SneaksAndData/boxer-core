use crate::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use crate::contracts::internal_token::encrypted_token::EncryptedToken;

/// Decrypts encrypted internal tokens into a claims collection.
pub trait Decryptor {
    /// Decrypts the provided token and returns extracted dynamic claims.
    fn decrypt(&self, token: &EncryptedToken) -> Result<DynamicClaimsCollection, anyhow::Error>;
}

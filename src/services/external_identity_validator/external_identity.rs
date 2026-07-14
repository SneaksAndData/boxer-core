use serde::{Deserialize, Serialize};

/// Struct that represents an external identity
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ExternalIdentity {
    /// The user ID extracted from the external identity provider
    user_id: String,

    /// The name of the external identity provider
    identity_provider: String,
}

impl ExternalIdentity {
    /// Creates a new instance of an external identity
    pub(super) fn new(identity_provider: String, user_id: String) -> Self {
        ExternalIdentity {
            user_id: user_id.to_lowercase(),
            identity_provider: identity_provider.to_lowercase(),
        }
    }

    #[cfg(test)]
    /// Constructor that should be used in tests. Should not be used in production code.
    pub fn for_test(identity_provider: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self::new(identity_provider.into(), user_id.into())
    }

    /// Returns the user ID of the external identity
    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }

    /// Returns the identity provider of the external identity
    pub fn identity_provider(&self) -> String {
        self.identity_provider.clone()
    }
}

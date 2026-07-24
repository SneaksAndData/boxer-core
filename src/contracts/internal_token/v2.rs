pub mod boxer_claims;
pub mod internal_token;

pub const PRINCIPAL_KEY: &str = "boxer.sneaksanddata.com/principal";
pub const SCHEMA_KEY: &str = "boxer.sneaksanddata.com/schema";
pub const SCHEMA_ID_KEY: &str = "boxer.sneaksanddata.com/schema-id";
pub const VALIDATOR_SCHEMA_ID_KEY: &str = "boxer.sneaksanddata.com/validator-schema-id";
pub const USER_ID_KEY: &str = "boxer.sneaksanddata.com/external-identity";
pub const IDENTITY_PROVIDER_KEY: &str = "boxer.sneaksanddata.com/identity-provider";
pub const AUDIT_EVENT: &str = "boxer.sneaksanddata.com/audit-event";

// The constants below to be moved in the service configuration file in the future.
pub const BOXER_ISSUER: &str = "boxer.sneaksanddata.com";
pub const BOXER_AUDIENCE: &str = "boxer.sneaksanddata.com";

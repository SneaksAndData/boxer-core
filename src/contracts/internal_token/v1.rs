pub mod boxer_claims;
pub mod token;

pub type Token = token::InternalToken;
pub type TokenBuilder = token::internal_token_builder::InternalTokenBuilder;

const PRINCIPAL_KEY: &str = "boxer.sneaksanddata.com/principal";
const SCHEMA_KEY: &str = "boxer.sneaksanddata.com/schema";
const SCHEMA_ID_KEY: &str = "boxer.sneaksanddata.com/schema-id";
const VALIDATOR_SCHEMA_ID_KEY: &str = "boxer.sneaksanddata.com/validator-schema-id";
const USER_ID_KEY: &str = "boxer.sneaksanddata.com/external-identity";
const IDENTITY_PROVIDER_KEY: &str = "boxer.sneaksanddata.com/identity-provider";

// The constants below to be moved in the service configuration file in the future.
const BOXER_ISSUER: &str = "boxer.sneaksanddata.com";
const BOXER_AUDIENCE: &str = "boxer.sneaksanddata.com";

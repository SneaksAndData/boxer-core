use crate::contracts::dynamic_claims_collection::{DynamicClaims, DynamicClaimsCollection};
use anyhow::Result;

pub mod v1;

const API_VERSION_KEY: &str = "boxer.sneaksanddata.com/api-version";

pub fn get_version(claims: &DynamicClaimsCollection) -> Result<String> {
    claims
        .get_claim(API_VERSION_KEY)
        .ok_or_else(|| anyhow::anyhow!("Missing api version"))
}

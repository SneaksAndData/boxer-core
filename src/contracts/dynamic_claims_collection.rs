use crate::contracts::internal_token::API_VERSION_KEY;
use anyhow::{anyhow, Result};
use josekit::jwt::JwtPayload;
use serde_json::Value;

pub trait DynamicClaims {
    fn get_claim(&self, key: &str) -> Option<String>;
    fn get_value(&self, key: &str) -> Option<Value>;

    // COVERAGE: Disable since the function is trivial
    #[cfg_attr(coverage, coverage(off))]
    fn get_version(&self) -> Result<String> {
        self.get_claim(API_VERSION_KEY).ok_or(anyhow!("Missing api version"))
    }
}

impl DynamicClaims for JwtPayload {
    // COVERAGE: Disable since the function is trivial
    #[cfg_attr(coverage, coverage(off))]
    fn get_claim(&self, key: &str) -> Option<String> {
        let value = self.claim(key)?;
        let value = value.as_str()?;
        Some(value.to_owned())
    }

    // COVERAGE: Disable since the function is trivial
    #[cfg_attr(coverage, coverage(off))]
    fn get_value(&self, key: &str) -> Option<Value> {
        let value = self.claim(key)?;
        Some(value.to_owned())
    }
}
pub type DynamicClaimsCollection = JwtPayload;

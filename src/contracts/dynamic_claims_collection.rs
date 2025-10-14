use josekit::jwt::JwtPayload;
use serde_json::Value;

pub trait DynamicClaims {
    fn get_claim(&self, key: &str) -> Option<String>;
    fn get_value(&self, key: &str) -> Option<Value>;
}

impl DynamicClaims for JwtPayload {
    fn get_claim(&self, key: &str) -> Option<String> {
        let value = self.claim(key)?;
        let value = value.as_str()?;
        Some(value.to_owned())
    }
    fn get_value(&self, key: &str) -> Option<Value> {
        let value = self.claim(key)?;
        Some(value.to_owned())
    }
}
pub type DynamicClaimsCollection = JwtPayload;

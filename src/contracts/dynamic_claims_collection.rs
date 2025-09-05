use josekit::jwt::JwtPayload;
use serde_json::Value;

pub type DynamicClaimsCollection = JwtPayload;
pub fn get_claim(claims: &DynamicClaimsCollection, key: &str) -> Option<String> {
    let value = claims.claim(key)?;
    let value = value.as_str()?;
    Some(value.to_owned())
}

pub fn get_value(claims: &DynamicClaimsCollection, key: &str) -> Option<Value> {
    let value = claims.claim(key)?;
    Some(value.to_owned())
}

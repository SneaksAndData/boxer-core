use serde_json::Value;
use std::collections::HashMap;

pub type DynamicClaimsCollection = HashMap<String, Value>;
pub fn get_claim(claims: &DynamicClaimsCollection, key: &str) -> Option<String> {
    let value = claims.get(key)?;
    let value = value.as_str()?;
    Some(value.to_owned())
}

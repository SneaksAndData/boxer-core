mod tests;

use kube::runtime::watcher::Config;
use kube::Resource;
use maplit::btreemap;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct ObjectOwnerMark {
    key: String,
    value: String,
}

impl ObjectOwnerMark {
    pub fn new(key: &str, value: &str) -> Self {
        ObjectOwnerMark {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn is_owned<S>(&self, object: &S) -> bool
    where
        S: Resource,
    {
        let labels = object.meta().labels.clone();
        let owner_label_value = labels.unwrap_or_default().get(&self.key).cloned();
        owner_label_value.map_or(false, |value| value == self.value)
    }

    pub fn get_resource_owner<S>(&self, object: &S) -> Option<String>
    where
        S: Resource,
    {
        object
            .meta()
            .labels
            .clone()
            .and_then(|labels| labels.get(&self.key).cloned())
    }

    pub fn get_owner_name(&self) -> String {
        self.key.clone()
    }
}

impl Into<Config> for &ObjectOwnerMark {
    fn into(self) -> Config {
        Config {
            label_selector: Some(format!("{}={}", self.key, self.value)),
            ..Default::default()
        }
    }
}

impl Into<BTreeMap<String, String>> for &ObjectOwnerMark {
    fn into(self) -> BTreeMap<String, String> {
        btreemap! { self.key.clone() => self.value.clone() }
    }
}

use crate::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use crate::services::base::upsert_repository::ReadOnlyRepository;
use crate::services::legacy::repositories::policy_repository::policy_document::PolicyDocument;
use async_trait::async_trait;
use cedar_policy::{Policy, PolicyId, PolicySet, PolicySetError};
use kube::runtime::watcher;
use log::{debug, warn};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PolicyRepositoryData {
    policy_set: RwLock<HashMap<String, PolicySet>>,
}

pub(crate) fn new() -> Arc<PolicyRepositoryData> {
    Arc::new(PolicyRepositoryData {
        policy_set: RwLock::new(HashMap::default()),
    })
}

#[async_trait]
impl ReadOnlyRepository<String, PolicySet> for PolicyRepositoryData {
    type ReadError = anyhow::Error;

    async fn get(&self, key: String) -> Result<PolicySet, Self::ReadError> {
        let guard = self.policy_set.read().await;
        match guard.get(&key) {
            Some(policy_set) => Ok(policy_set.clone()),
            None => Err(anyhow::anyhow!("Policy set not found for key: {}", key)),
        }
    }
}

#[async_trait]
impl ResourceUpdateHandler<PolicyDocument> for PolicyRepositoryData {
    async fn handle_update(&self, event: Result<PolicyDocument, watcher::Error>) -> () {
        match event {
            Err(err) => warn!("Error while fetching policy: {:?}", err),
            Ok(event) => {
                debug!("Received policy update: {:?}", event);
                let policy = Policy::from_str(&event.spec.policies);
                match policy {
                    Err(err) => warn!("Failed to parse policy set: {:?}", err),
                    Ok(new_policy) => {
                        // We use here unwrap because the name is guaranteed to be present by Kubernetes
                        // and PolicyId::from_str return Infallible as an error
                        let new_id = PolicyId::from_str(&event.metadata.name.clone().unwrap()).unwrap();
                        let new_policy = new_policy.new_id(new_id);
                        let mut guard = self.policy_set.write().await;
                        insert_or_replace(&mut guard, event.spec.schema.clone(), new_policy.clone()).unwrap_or_else(
                            |err| {
                                warn!("Failed to insert or replace policy: {:?}", err);
                            },
                        );
                    }
                }
            }
        }
    }
}

fn insert_or_replace(
    map: &mut HashMap<String, PolicySet>,
    key: String,
    new_policy: Policy,
) -> Result<(), PolicySetError> {
    match map.get_mut(&key) {
        Some(existing) => {
            let result = existing.add(new_policy.clone());
            if let Err(PolicySetError::AlreadyDefined(_)) = result {
                warn!(
                    "Policy with ID {:?} already exists in the set, overwriting.",
                    new_policy.id()
                );
                let _ = existing.remove_static(new_policy.id().clone())?;
                existing.add(new_policy)?;
                return Ok(());
            }
            result
        }
        None => {
            let mut new_set = PolicySet::new();
            new_set.add(new_policy)?;
            map.insert(key, new_set);
            Ok(())
        }
    }
}

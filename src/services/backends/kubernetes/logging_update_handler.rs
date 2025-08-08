#[cfg(not(test))]
use log::{debug, warn};

#[cfg(test)]
use std::{println as warn, println as debug};

use crate::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use async_trait::async_trait;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::watcher;
use kube::Resource;

use std::fmt::Debug;

pub struct LoggingUpdateHandler;

#[async_trait]
impl<T> ResourceUpdateHandler<T> for LoggingUpdateHandler
where
    T: Resource + Debug + Send + Sync + 'static,
{
    async fn handle_update(&self, event: Result<T, watcher::Error>) -> () {
        if let Err(e) = event {
            warn!("Error processing event: {}", e);
            return;
        }
        if let Ok(event) = event {
            let metadata: &ObjectMeta = event.meta();
            debug!(
                "Received event for resource: {:?} in namespace {:?}",
                metadata.name, metadata.namespace
            );
        }
    }
}

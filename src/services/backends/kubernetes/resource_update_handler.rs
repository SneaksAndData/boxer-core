pub mod composed_update_handler;
pub mod logging_update_handler;

use async_trait::async_trait;
use kube::Resource;
use kube::runtime::watcher;

#[async_trait]
pub trait ResourceUpdateHandler<S>: Send + Sync
where
    S: Resource + Send + Sync,
{
    async fn handle_update(&self, result: &Result<S, watcher::Error>);
}

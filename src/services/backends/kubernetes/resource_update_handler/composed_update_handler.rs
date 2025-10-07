use crate::services::backends::kubernetes::resource_update_handler::ResourceUpdateHandler;
use async_trait::async_trait;
use kube::Resource;
use std::fmt::Debug;

pub struct ComposedUpdateHandler<T> {
    handlers: Vec<Box<dyn ResourceUpdateHandler<T>>>,
}

impl<T> ComposedUpdateHandler<T> {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn add_handler(mut self, handler: Box<dyn ResourceUpdateHandler<T>>) -> Self {
        self.handlers.push(handler);
        self
    }
}

#[async_trait]
impl<T> ResourceUpdateHandler<T> for ComposedUpdateHandler<T>
where
    T: Resource + Debug + Send + Sync + 'static,
{
    async fn handle_update(&self, result: &Result<T, kube::runtime::watcher::Error>) -> () {
        for handler in &self.handlers {
            handler.handle_update(result).await;
        }
    }
}

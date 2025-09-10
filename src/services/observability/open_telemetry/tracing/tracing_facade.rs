use crate::services::base::upsert_repository::ReadOnlyRepository;
use crate::services::observability::open_telemetry::tracing::{start_trace, ErrorExt};
use async_trait::async_trait;
use opentelemetry::context::FutureExt;
use std::marker::PhantomData;
use std::sync::Arc;

pub trait WithTracingFacade<Key, Value> {
    fn with_tracing(
        self: Arc<Self>,
        span_name: String,
    ) -> Arc<dyn ReadOnlyRepository<Key, Value, ReadError = anyhow::Error>>
    where
        Self: Sized + Send + Sync + 'static;
}

impl<Repo, Key, Value> WithTracingFacade<Key, Value> for Repo
where
    Repo: ReadOnlyRepository<Key, Value, ReadError = anyhow::Error> + Send + Sync + 'static,
    Key: Send + Sync + 'static,
    Value: Send + Sync + 'static,
{
    fn with_tracing(
        self: Arc<Self>,
        span_name: String,
    ) -> Arc<dyn ReadOnlyRepository<Key, Value, ReadError = anyhow::Error>> {
        Arc::new(TracingFacade {
            span_name,
            underlying: self,
            _p: Default::default(),
        })
    }
}

struct TracingFacade<Repo, Key, Value>
where
    Repo: ReadOnlyRepository<Key, Value, ReadError = anyhow::Error>,
    Key: Send + Sync + 'static,
    Value: Send + Sync + 'static,
{
    span_name: String,
    underlying: Arc<Repo>,
    _p: PhantomData<(Key, Value)>,
}

#[async_trait]
impl<Repo, Key, Value> ReadOnlyRepository<Key, Value> for TracingFacade<Repo, Key, Value>
where
    Repo: ReadOnlyRepository<Key, Value, ReadError = anyhow::Error>,
    Key: Send + Sync + 'static,
    Value: Send + Sync + 'static,
{
    type ReadError = anyhow::Error;

    async fn get(&self, key: Key) -> Result<Value, Self::ReadError> {
        let cx = start_trace(&self.span_name, None);
        self.underlying.get(key).with_context(cx.clone()).await.stop_trace(cx)
    }
}

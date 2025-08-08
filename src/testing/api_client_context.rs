use crate::testing::temp_namespace_context::TempNamespaceContext;
use k8s_openapi::NamespaceResourceScope;
use kube::{Api, Client};
use test_context::AsyncTestContext;

pub struct ApiClientContext<R> {
    pub api: Api<R>,
    namespace_context: TempNamespaceContext,
    pub client: Client,
}

impl<R> ApiClientContext<R>
where
    R: kube::Resource<Scope = NamespaceResourceScope> + Clone + Send + Sync + 'static,
    R::DynamicType: Default,
{
    pub fn new_api<T>(&self) -> Api<T>
    where
        T: kube::Resource<Scope = NamespaceResourceScope> + Clone + Send + Sync + 'static,
        T::DynamicType: Default,
    {
        Api::namespaced(self.client.clone(), &self.namespace_context.namespace)
    }

    pub fn namespace(&self) -> &str {
        &self.namespace_context.namespace
    }

    pub fn config(&self) -> &kube::Config {
        &self.namespace_context.config
    }

    pub fn api(&self) -> &Api<R> {
        &self.api
    }
}

impl<R> AsyncTestContext for ApiClientContext<R>
where
    R: kube::Resource<Scope = NamespaceResourceScope> + Clone + Send + Sync + 'static,
    R::DynamicType: Default,
{
    async fn setup() -> Self {
        let namespace_context = TempNamespaceContext::setup().await;
        let client = Client::try_from(namespace_context.config.clone()).expect("Failed to create Kubernetes client");
        let api: Api<R> = Api::namespaced(client.clone(), &namespace_context.namespace);
        ApiClientContext {
            api,
            client,
            namespace_context,
        }
    }
}

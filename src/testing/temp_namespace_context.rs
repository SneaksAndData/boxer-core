use crate::testing::{create_namespace, get_kubeconfig};
use kube::Config;
use test_context::AsyncTestContext;

pub struct TempNamespaceContext {
    pub namespace: String,
    pub config: Config,
}

impl AsyncTestContext for TempNamespaceContext {
    async fn setup() -> Self {
        let namespace = create_namespace().await.expect("Failed to create namespace");
        let config = get_kubeconfig().await.expect("Failed to create config");
        TempNamespaceContext { namespace, config }
    }
}

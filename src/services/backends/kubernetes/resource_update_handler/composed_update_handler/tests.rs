use crate::services::backends::kubernetes::resource_update_handler::ResourceUpdateHandler;
use crate::services::backends::kubernetes::resource_update_handler::composed_update_handler::ComposedUpdateHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::runtime::watcher;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

struct TestResourceUpdateHandler {
    sucessfull_calls: Arc<AtomicUsize>,
    failed_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl ResourceUpdateHandler<ConfigMap> for TestResourceUpdateHandler {
    async fn handle_update(&self, result: &Result<ConfigMap, watcher::Error>) {
        match result {
            Ok(result) => {
                result.metadata.name.as_ref().map(|name| {
                    if name == "bad-configmap" {
                        println!("Received bad ConfigMap: {}", name);
                        self.failed_calls.fetch_add(1, SeqCst);
                        Err(anyhow!("Test error for bad-configmap"))
                    } else {
                        println!("Received good ConfigMap: {}", name);
                        self.sucessfull_calls.fetch_add(1, SeqCst);
                        Ok("Test success for bad-configmap")
                    }
                });
            }
            Err(err) => panic!("Test should not receive errors: {:?}", err),
        }
    }
}

#[tokio::test]
/// Validates that the ComposedUpdateHandler does not stop processing other handlers on error
async fn test_error_handling() {
    let sucessfull_calls = Arc::new(AtomicUsize::new(0));
    let failed_calls = Arc::new(AtomicUsize::new(0));
    let handler = ComposedUpdateHandler::new()
        .add_handler(Box::new(TestResourceUpdateHandler {
            sucessfull_calls: sucessfull_calls.clone(),
            failed_calls: failed_calls.clone(),
        }))
        .add_handler(Box::new(TestResourceUpdateHandler {
            sucessfull_calls: sucessfull_calls.clone(),
            failed_calls: failed_calls.clone(),
        }))
        .add_handler(Box::new(TestResourceUpdateHandler {
            sucessfull_calls: sucessfull_calls.clone(),
            failed_calls: failed_calls.clone(),
        }));

    let cms = vec![
        ConfigMap {
            metadata: kube::api::ObjectMeta {
                name: Some("good-configmap".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        ConfigMap {
            metadata: kube::api::ObjectMeta {
                name: Some("bad-configmap".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        ConfigMap {
            metadata: kube::api::ObjectMeta {
                name: Some("another-good-configmap".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    for cm in cms {
        handler.handle_update(&Ok(cm)).await;
    }

    assert_eq!(sucessfull_calls.load(SeqCst), 2 * 3); // Each good configmap should be counted by each handler
    assert_eq!(failed_calls.load(SeqCst), 1 * 3); // Each bad configmap should be counted by each handler
}

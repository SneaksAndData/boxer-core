use crate::services::backends::kubernetes::repositories::schema_repository::schema_document::{
    SchemaDocument, SchemaDocumentSpec,
};
use crate::testing::api_extensions::WaitForResource;
use crate::testing::spin_lock_kubernetes_resource_manager_context::SpinLockKubernetesResourceManagerTestContext;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::reflector::ObjectRef;
use maplit::btreemap;
use std::time::Duration;
use test_context::test_context;

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

fn create_object(name: &str, namespace: &str, resource_version: String) -> SchemaDocument {
    SchemaDocument {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            labels: Some(btreemap! {
                "repository.boxer.io/test".to_string() => "test-label".to_string(),
            }),
            resource_version: Some(resource_version),
            ..Default::default()
        },
        spec: SchemaDocumentSpec {
            schema: "{}".to_string(),
            active: true,
        },
        ..Default::default()
    }
}

#[test_context(SpinLockKubernetesResourceManagerTestContext)]
#[tokio::test]
async fn test_create_object(ctx: &mut SpinLockKubernetesResourceManagerTestContext) {
    // Arrange
    let name = "test-object";
    let resource = create_object(name, &ctx.config.namespace, Default::default());
    let object_ref = &ObjectRef::from(&resource);

    // Act
    let created_object = ctx.manager.upsert(object_ref, resource).await;

    assert!(created_object.is_ok());
    let created_object = created_object.unwrap();
    let labels = created_object.metadata.labels.unwrap();

    assert!(labels.contains_key(ctx.config.listener_config.label_selector_key.as_str()));
    assert_eq!(
        labels.get(ctx.config.listener_config.label_selector_key.as_str()),
        Some(&ctx.config.listener_config.label_selector_value)
    );
    assert!(labels.contains_key("repository.boxer.io/test"));
}

#[test_context(SpinLockKubernetesResourceManagerTestContext)]
#[tokio::test]
async fn test_patch_unexisted_object(ctx: &mut SpinLockKubernetesResourceManagerTestContext) {
    // Arrange
    let name = "test-object";
    let resource = create_object(name, &ctx.config.namespace, Default::default());
    let object_ref = &ObjectRef::from(&resource);
    // Simulate a parallel update
    let mut created_object = ctx.manager.upsert(object_ref, resource).await.unwrap();
    created_object.metadata.managed_fields = None;
    created_object.spec.active = false;

    let _ = ctx.manager.upsert(object_ref, created_object.clone()).await;

    // Act
    let operation_status = ctx
        .manager
        .upsert(object_ref, created_object)
        .await
        .unwrap_err()
        .to_string();

    // Assert
    assert_eq!(operation_status, "Conflict error occurred");
}

#[test_context(SpinLockKubernetesResourceManagerTestContext)]
#[tokio::test]
async fn test_get_object(ctx: &mut SpinLockKubernetesResourceManagerTestContext) {
    // Arrange
    let name = "test-object";
    let resource = create_object(name, &ctx.config.namespace, Default::default());
    let mut object_ref = ObjectRef::from(&resource);
    object_ref.namespace = Some(ctx.config.namespace.clone());

    // Simulate a parallel update
    let _ = ctx.manager.upsert(&object_ref, resource).await.unwrap();

    ctx.api_context
        .api
        .wait_for_creation(name.to_string(), ctx.manager.namespace(), DEFAULT_TEST_TIMEOUT)
        .await;

    // Act
    let operation_status = ctx.manager.get(&object_ref);

    // Assert
    assert!(operation_status.is_ok());
}

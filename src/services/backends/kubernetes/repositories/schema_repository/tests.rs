use super::*;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status::{Deleted, NotFound, NotOwned};
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::not_found_details::NotFoundDetails;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::owner_conflict_details::OwnerConflictDetails;
use crate::services::backends::kubernetes::repositories::TryIntoObjectRef;
use crate::services::backends::kubernetes::repositories::schema_repository::test_reduced_schema::reduced_schema;
use crate::services::backends::kubernetes::repositories::schema_repository::test_schema::schema;
use crate::testing::api_extensions::{WaitForDelete, WaitForResource};
use crate::testing::spin_lock_kubernetes_resource_manager_context::SpinLockKubernetesResourceManagerTestContext;
use assert_matches::assert_matches;
use kube::Api;
use kube::api::PostParams;
use kube::runtime::reflector::ObjectRef;
use maplit::btreemap;
use std::collections::BTreeMap;
use std::time::Duration;
use test_context::{AsyncTestContext, test_context};

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

struct KubernetesSchemaRepositoryTest {
    repository: Arc<SchemaRepository>,
    api: Api<SchemaDocument>,
    namespace: String,
    label: String,
}

impl AsyncTestContext for KubernetesSchemaRepositoryTest {
    async fn setup() -> KubernetesSchemaRepositoryTest {
        let parent = SpinLockKubernetesResourceManagerTestContext::setup().await;
        let label = parent.config.owner_mark.get_owner_name().clone();
        let repository = Arc::new(KubernetesRepository {
            resource_manager: parent.manager,
            operation_timeout: parent.config.operation_timeout,
        });
        Self {
            repository,
            api: parent.api_context.api,
            namespace: parent.config.namespace.clone(),
            label,
        }
    }
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_create_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema";
    let schema_str = schema();
    let schema_fragment = SchemaFragment::from_json_value(schema_str).expect("Failed to create schema fragment");

    let before = ctx.repository.get(name.to_string()).await;

    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");

    // Act
    ctx.api
        .wait_for_creation(name.to_string(), ctx.namespace.to_string(), DEFAULT_TEST_TIMEOUT)
        .await;

    let after = ctx.repository.get(name.to_string()).await;

    // Assert
    assert!(before.is_err());
    assert!(after.is_ok());
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_update_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema";
    let schema_fragment = SchemaFragment::from_json_value(schema()).expect("Failed to create schema fragment");
    let reduced_schema_fragment =
        SchemaFragment::from_json_value(reduced_schema()).expect("Failed to create schema fragment");

    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");
    ctx.api
        .wait_for_creation(name.to_string(), ctx.namespace.to_string(), DEFAULT_TEST_TIMEOUT)
        .await;
    let before = ctx.repository.get(name.to_string()).await.unwrap();

    // Act
    ctx.repository
        .upsert(name.to_string(), reduced_schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");

    ctx.api
        .wait_for_creation(name.to_string(), ctx.namespace.to_string(), DEFAULT_TEST_TIMEOUT)
        .await;

    let after = ctx.repository.get(name.to_string()).await.unwrap();

    // Assert
    assert_ne!(before.to_json_string().unwrap(), after.to_json_string().unwrap());
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_delete_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-delete-schema";
    let schema_fragment = SchemaFragment::from_json_value(schema()).expect("Failed to create schema fragment");

    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");
    ctx.api
        .wait_for_creation(name.to_string(), ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;

    // Act
    ctx.repository
        .delete(name.to_string())
        .await
        .expect("Failed to upsert schema");
    ctx.api
        .wait_for_deletion::<SchemaDocument>(name.to_string(), ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;

    let after = ctx.repository.get(name.to_string()).await;

    // Assert
    assert_matches!(
        after.unwrap_err(),
        Deleted(NotFoundDetails {
            name: _,
            namespace: _,
            resource_type: rt,
        }) if rt == "SchemaDocument"
    );
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_schema_name(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "!@test-name-schema--#".to_string();
    let or: ObjectRef<SchemaDocument> = name.clone().try_into_object_ref(ctx.namespace.clone()).unwrap();
    let schema_fragment = SchemaFragment::from_json_value(schema()).expect("Failed to create schema fragment");

    // Act
    ctx.repository
        .upsert(name.clone(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");
    ctx.api
        .wait_for_creation(or.name.clone(), ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;

    let after = ctx.repository.get(name.clone()).await;

    // Assert
    assert!(after.is_ok());
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_schema_no_owner_conflict(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-not-owned-schema";
    let schema_str = schema();
    let schema_fragment = SchemaFragment::from_json_value(schema_str).expect("Failed to create schema fragment");

    let resource = SchemaDocument {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(ctx.namespace.clone()),
            ..Default::default()
        },
        spec: SchemaDocumentSpec::default(),
    };

    // Act
    let pp = PostParams {
        field_manager: Some("test-manager".to_string()),
        ..Default::default()
    };
    ctx.api.create(&pp, &resource).await.unwrap();
    ctx.api
        .wait_for_creation(name.to_string(), ctx.namespace.to_string(), DEFAULT_TEST_TIMEOUT)
        .await;

    let insertion_result = ctx.repository.upsert(name.to_string(), schema_fragment.clone()).await;

    // Assert
    assert_matches!(
        insertion_result,
        Err(NotOwned(OwnerConflictDetails {
            name: _,
            namespace: _,
            current_owner: None,
            resource_type: _,
        }))
    );
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_schema_other_owner_conflict(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-not-owned-schema";
    let schema_str = schema();
    let schema_fragment = SchemaFragment::from_json_value(schema_str).expect("Failed to create schema fragment");
    let owner = "test-owner".to_string();
    let resource = SchemaDocument {
        metadata: ObjectMeta {
            labels: Some(btreemap! {ctx.label.clone() => owner.clone()}),
            name: Some(name.to_string()),
            namespace: Some(ctx.namespace.clone()),
            ..Default::default()
        },
        spec: SchemaDocumentSpec::default(),
    };

    // Act
    let pp = PostParams {
        field_manager: Some(owner.clone()),
        ..Default::default()
    };
    ctx.api.create(&pp, &resource).await.unwrap();
    ctx.api
        .wait_for_creation(name.to_string(), ctx.namespace.to_string(), DEFAULT_TEST_TIMEOUT)
        .await;

    let insertion_result = ctx.repository.upsert(name.to_string(), schema_fragment.clone()).await;

    // Assert
    assert_matches!(
        insertion_result,
        Err(NotOwned(OwnerConflictDetails {
            name: _,
            namespace: _,
            current_owner: Some(_),
            resource_type: _,
        }))
    );
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_schema_delete_not_owned_resource(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-not-owned-schema";
    let owner = "test-owner".to_string();
    let resource = SchemaDocument {
        metadata: ObjectMeta {
            labels: Some(BTreeMap::from([(ctx.label.clone(), owner.clone())])),
            name: Some(name.to_string()),
            namespace: Some(ctx.namespace.clone()),
            ..Default::default()
        },
        spec: SchemaDocumentSpec::default(),
    };

    // Act
    let pp = PostParams {
        field_manager: Some("test-manager".to_string()),
        ..Default::default()
    };
    ctx.api.create(&pp, &resource).await.unwrap();
    ctx.api
        .wait_for_creation(name.to_string(), ctx.namespace.to_string(), DEFAULT_TEST_TIMEOUT)
        .await;

    let after = ctx.repository.delete(name.to_string()).await;

    // Assert
    assert_eq!(
        after.as_ref().unwrap_err().to_string(),
        format!(
            "Owner conflict: Resource of kind 'SchemaDocument' with name: '{}',  namespace '{}' is not owned by us, current owner: test-owner",
            name, ctx.namespace
        )
    );
    assert_matches!(
        after,
        Err(NotOwned(OwnerConflictDetails {
            name: _,
            namespace: _,
            current_owner: Some(_),
            resource_type: _,
        }))
    );
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_not_existing_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "never-created-schema";

    // Act
    let after = ctx.repository.get(name.to_string()).await;

    // Assert
    assert_eq!(
        after.as_ref().unwrap_err().to_string(),
        format!(
            "Resource not found: Resource of kind 'SchemaDocument' with name: '{}',  namespace '{}' not found",
            name, ctx.namespace
        )
    );
    assert_matches!(
        after.unwrap_err(),
        NotFound(NotFoundDetails {
            name: _,
            namespace: _,
            resource_type: rt,
        }) if rt == "SchemaDocument"
    );
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_exists(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema".to_string();
    let or: ObjectRef<SchemaDocument> = name.clone().try_into_object_ref(ctx.namespace.clone()).unwrap();
    let schema_fragment = SchemaFragment::from_json_value(schema()).expect("Failed to create schema fragment");

    // Act
    ctx.repository
        .upsert(name.clone(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");
    ctx.api
        .wait_for_creation(or.name.clone(), ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;

    let after = ctx.repository.exists(name.clone()).await;

    // Assert
    assert_eq!(after.unwrap(), true);
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_not_exists(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "never-created-schema";

    // Act
    let after = ctx.repository.exists(name.to_string()).await;

    // Assert
    assert_eq!(after.unwrap(), false);
}

use super::*;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::NotFoundDetails;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status::Deleted;
use crate::services::backends::kubernetes::repositories::schema_repository::test_reduced_schema::reduced_schema;
use crate::services::backends::kubernetes::repositories::schema_repository::test_schema::schema;
use crate::testing::api_extensions::{WaitForDelete, WaitForResource};
use crate::testing::spin_lock_kubernetes_resource_manager_context::SpinLockKubernetesResourceManagerTestContext;
use assert_matches::assert_matches;
use kube::Api;
use std::time::Duration;
use test_context::{AsyncTestContext, test_context};

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

struct KubernetesSchemaRepositoryTest {
    repository: Arc<SchemaRepository>,
    api: Api<SchemaDocument>,
    namespace: String,
}

impl AsyncTestContext for KubernetesSchemaRepositoryTest {
    async fn setup() -> KubernetesSchemaRepositoryTest {
        let parent = SpinLockKubernetesResourceManagerTestContext::setup().await;
        let repository = Arc::new(KubernetesRepository {
            resource_manager: parent.manager,
            operation_timeout: parent.config.operation_timeout,
        });
        Self {
            repository,
            api: parent.api_context.api,
            namespace: parent.config.namespace.clone(),
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
    // ctx.api
    //     .wait_for_creation(&ObjectRef::new(name), DEFAULT_TEST_TIMEOUT)
    //     .await;

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
    // ctx.api
    //     .wait_for_creation(&ObjectRef::new(name), DEFAULT_TEST_TIMEOUT)
    //     .await;
    let before = ctx.repository.get(name.to_string()).await.unwrap();

    // Act
    ctx.repository
        .upsert(name.to_string(), reduced_schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");

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
    assert_matches!(after.unwrap_err(), Deleted(NotFoundDetails { name: _, namespace: _ }));
}

use super::*;
use crate::services::backends::kubernetes::repositories::schema_repository::test_schema::schema;
use crate::services::base::upsert_repository::UpsertRepository;
use crate::testing::{create_namespace, get_kubeconfig};
use std::sync::Arc;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

struct KubernetesSchemaRepositoryTest {
    repository: Arc<KubernetesSchemaRepository>,
    schema_str: String,
}

static LABEL_SELECTOR_KEY: &str = "repository.boxer.io/type";
const LABEL_SELECTOR_VALUE: &str = "schema";

impl AsyncTestContext for KubernetesSchemaRepositoryTest {
    async fn setup() -> KubernetesSchemaRepositoryTest {
        let namespace = create_namespace().await.expect("Failed to create namespace");
        let config = get_kubeconfig().await.expect("Failed to create config");

        let config = KubernetesResourceManagerConfig {
            namespace: namespace.clone(),
            label_selector_key: LABEL_SELECTOR_KEY.to_string(),
            label_selector_value: LABEL_SELECTOR_VALUE.to_string(),
            lease_name: "schemas".to_string(),
            kubeconfig: config,
            lease_duration: Duration::from_secs(5),
            renew_deadline: Duration::from_secs(3),
            claimant: "boxer".to_string(),
        };

        let repository = KubernetesSchemaRepository::start(config)
            .await
            .expect("Failed to start repository");

        KubernetesSchemaRepositoryTest {
            repository: Arc::new(repository),
            schema_str: serde_json::to_string(&schema()).expect("Failed to serialize schema to JSON"),
        }
    }

    async fn teardown(self) {
        // do nothing
    }
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_create_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema";
    let schema_fragment = SchemaFragment::from_json_str(&ctx.schema_str).expect("Failed to create schema fragment");

    // Act
    let before = ctx.repository.get(name.to_string()).await;

    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");

    let after = ctx.repository.get(name.to_string()).await;

    // Assert
    assert!(before.is_err());
    assert!(after.is_ok());
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_delete_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema";
    let schema_fragment = SchemaFragment::from_json_str(&ctx.schema_str).expect("Failed to create schema fragment");

    // Act
    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");
    let before = ctx.repository.get(name.to_string()).await;

    ctx.repository
        .delete(name.to_string())
        .await
        .expect("Failed to upsert schema");

    let after = ctx.repository.get(name.to_string()).await;

    // Assert
    assert!(before.is_ok());
    assert!(after.is_err());
}

#[test_context(KubernetesSchemaRepositoryTest)]
#[tokio::test]
async fn test_update_schema(ctx: &mut KubernetesSchemaRepositoryTest) {
    // Arrange
    let name = "test-schema";
    let schema_fragment = SchemaFragment::from_json_str(&ctx.schema_str).expect("Failed to create schema fragment");
    let reduced_schema_fragment =
        SchemaFragment::from_json_str(&ctx.schema_str).expect("Failed to create schema fragment");

    // Act
    ctx.repository
        .upsert(name.to_string(), schema_fragment.clone())
        .await
        .expect("Failed to upsert schema");

    let before = ctx.repository.get(name.to_string()).await.unwrap();

    ctx.repository
        .upsert(name.to_string(), reduced_schema_fragment)
        .await
        .expect("Failed to upsert schema");

    let after = ctx.repository.get(name.to_string()).await;

    // Assert
    assert_ne!(
        before.to_json_string().unwrap(),
        after.unwrap().to_json_string().unwrap()
    );
}

use super::*;
use crate::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use crate::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use crate::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcherRunner;
use crate::services::legacy::repositories::action_repository::action_discovery_document::ActionDiscoveryDocument;
use crate::services::legacy::repositories::action_repository::action_route_registration::ActionRouteRegistration;
use crate::services::legacy::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::legacy::repositories::action_repository::schema_bound_action_route_registration::SchemaBoundActionSetRegistration;
use crate::services::legacy::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::service_provider::ServiceProvider;
use crate::services::validation_service::request_context::RequestContext;
use crate::testing::api_extensions::{WaitForDelete, WaitForResource};
use crate::testing::spin_lock_kubernetes_resource_manager_context::GenericKubernetesResourceManagerTestContext;
use cedar_policy::EntityUid;
use kube::Api;
use std::sync::Arc;
use std::time::Duration;
use test_context::{AsyncTestContext, test_context};

const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(10);

struct KubernetesActionRepositoryTest {
    repository: Arc<ActionDataRepository>,
    api: Api<ActionDiscoveryDocument>,
    namespace: String,
    lookup: ReadOnlyRepositoryBackend<
        SchemaBoundedTrieRepositoryData<RequestSegment>,
        ActionDiscoveryDocument,
        (String, Vec<RequestSegment>),
        EntityUid,
    >,
}

impl AsyncTestContext for KubernetesActionRepositoryTest {
    async fn setup() -> Self {
        let parent = GenericKubernetesResourceManagerTestContext::setup().await;
        let owner_mark = parent.config.owner_mark.clone();
        let operation_timeout = parent.config.operation_timeout.clone();
        let config = KubernetesResourceManagerConfig {
            namespace: parent.config.namespace.clone(),
            kubeconfig: parent.config.kubeconfig.clone(),
            owner_mark,
            operation_timeout: operation_timeout.clone(),
        };
        let lookup_trie = Arc::new(SchemaBoundedTrieRepositoryData::<RequestSegment>::new());
        let mut lookup = ReadOnlyRepositoryBackend::new(lookup_trie.clone(), lookup_trie.clone());
        lookup.start(config).await.unwrap();
        let repository = Arc::new(
            KubernetesRepository::start(parent.manager, parent.config.operation_timeout.clone())
                .await
                .unwrap(),
        );

        Self {
            repository,
            api: parent.api_context.api,
            namespace: parent.config.namespace.clone(),
            lookup,
        }
    }
}

#[test_context(KubernetesActionRepositoryTest)]
#[tokio::test]
async fn test_create_action_document(ctx: &mut KubernetesActionRepositoryTest) {
    // Arrange

    insert_schema_document(
        ctx,
        "action-discovery-document",
        "api/v1/resources/{resourceId}",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;

    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources/resource".to_string(),
        "GET".to_string(),
    );
    let key = request_context.try_into().unwrap();

    let lookup_trie = ctx.lookup.get();
    let result = lookup_trie.get(("schema".to_string(), key)).await;

    // Assert
    assert!(result.is_ok());
}

#[test_context(KubernetesActionRepositoryTest)]
#[tokio::test]
async fn test_multiple_actions(ctx: &mut KubernetesActionRepositoryTest) {
    // Arrange
    insert_schema_document(
        ctx,
        "action-discovery-document-first",
        "api/v1/resources/resource/{resource_id}",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;
    insert_schema_document(
        ctx,
        "action-discovery-document-second",
        "api/v1/resources/{resource_id}",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;
    let lookup_trie = ctx.lookup.get();

    // Act
    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources/1".to_string(),
        "GET".to_string(),
    );
    let key: Vec<RequestSegment> = request_context.try_into().unwrap();
    let first_result = lookup_trie.get(("schema".to_string(), key)).await;

    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources/2".to_string(),
        "GET".to_string(),
    );
    let key = request_context.try_into().unwrap();

    let second_result = lookup_trie.get(("schema".to_string(), key)).await;

    // Assert
    assert!(first_result.is_ok(), "First result was an error: {:?}", first_result);
    assert!(second_result.is_ok(), "Second result was an error: {:?}", second_result);
}

#[test_context(KubernetesActionRepositoryTest)]
#[tokio::test]
async fn test_remove(ctx: &mut KubernetesActionRepositoryTest) {
    // Arrange
    insert_schema_document(
        ctx,
        "action-discovery-document-first",
        "api/v1/resources",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;

    insert_schema_document(
        ctx,
        "action-discovery-document-second",
        "api/v2/resources",
        "PhotoApp::Photo::\"vacationPhoto.jpg\"",
    )
    .await;

    let lookup_trie = ctx.lookup.get();

    ctx.repository
        .delete(("schema".to_string(), "action-discovery-document-first".to_string()))
        .await
        .unwrap();

    let key = format!("{}-{}", "schema", "action-discovery-document-first");
    ctx.api
        .wait_for_deletion::<ActionDiscoveryDocument>(key, ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;

    // Act
    let request_context = RequestContext::new(
        "https://www.example.com/api/v1/resources".to_string(),
        "GET".to_string(),
    );
    let key: Vec<RequestSegment> = request_context.try_into().unwrap();
    let first_result = lookup_trie.get(("schema".to_string(), key)).await;

    // Assert
    assert!(first_result.is_err());
}

async fn insert_schema_document(ctx: &KubernetesActionRepositoryTest, name: &str, route: &str, action_uid: &str) {
    let registration = SchemaBoundActionSetRegistration {
        hostname: "www.example.com".to_string(),
        routes: vec![ActionRouteRegistration {
            method: "GET".to_string(),
            route_template: route.to_string(),
            action_uid: action_uid.to_string(),
        }],
        schema: "schema".to_string(),
    };

    ctx.repository
        .upsert(("schema".to_string(), name.to_string()), registration)
        .await
        .expect("Failed to upsert schema");

    let key = format!("{}-{}", "schema", name);
    ctx.api
        .wait_for_creation(key, ctx.namespace.clone(), DEFAULT_TEST_TIMEOUT)
        .await;
}

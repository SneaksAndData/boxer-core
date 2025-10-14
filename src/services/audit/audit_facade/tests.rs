use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;

use crate::services::audit::AuditService;
use crate::services::audit::audit_facade::WithAuditFacade;
use crate::services::audit::audit_facade::to_audit_record::ToAuditRecord;
use crate::services::audit::events::authorization_audit_event::AuthorizationAuditEvent;
use crate::services::audit::events::resource_delete_audit_event::ResourceDeleteAuditEvent;
use crate::services::audit::events::resource_modification_audit_event::ResourceModificationAuditEvent;
use crate::services::audit::events::token_validation_event::TokenValidationEvent;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::base::upsert_repository::{
    CanDelete, ReadOnlyRepository, UpsertRepository, UpsertRepositoryWithDelete,
};

#[tokio::test]
async fn upsert_success_records_modification_event() {
    let repo = Arc::new(InMemoryRepo::new());
    let audit = Arc::new(MockAuditService::default());
    let audited = repo.with_audit(audit.clone());

    let v = audited
        .upsert("k1".to_string(), TestValue { data: "v1".to_string() })
        .await
        .unwrap();

    assert_eq!(v.data, "v1");
    assert_eq!(audit.count_mod(), 1);
    assert_eq!(audit.count_del(), 0);
}

#[tokio::test]
async fn upsert_failure_records_modification_event() {
    let repo = Arc::new(InMemoryRepo::new());
    let audit = Arc::new(MockAuditService::default());
    let audited = repo.with_audit(audit.clone());

    let res = audited
        .upsert("forbidden".to_string(), TestValue { data: "x".to_string() })
        .await;
    assert!(res.is_err());
    assert_eq!(audit.count_mod(), 1);
    assert_eq!(audit.count_del(), 0);
}

#[tokio::test]
async fn delete_success_records_deletion_event() {
    let repo = Arc::new(InMemoryRepo::new());
    repo.upsert("k1".to_string(), TestValue { data: "v1".to_string() })
        .await
        .unwrap();
    let audit = Arc::new(MockAuditService::default());
    let audited = repo.clone().with_audit(audit.clone());

    audited.delete("k1".to_string()).await.unwrap();
    assert_eq!(audit.count_del(), 1);
    assert_eq!(audit.count_mod(), 0);
}

#[tokio::test]
async fn delete_failure_records_deletion_event() {
    let repo = Arc::new(InMemoryRepo::new());
    let audit = Arc::new(MockAuditService::default());
    let audited = repo.clone().with_audit(audit.clone());

    let res = audited.delete("absent".to_string()).await;
    assert!(res.is_err());
    assert_eq!(audit.count_del(), 1);
}

#[tokio::test]
async fn get_and_exists_do_not_record_events() {
    let repo = Arc::new(InMemoryRepo::new());
    repo.upsert("k1".to_string(), TestValue { data: "v1".to_string() })
        .await
        .unwrap();
    let audit = Arc::new(MockAuditService::default());
    let audited = repo.clone().with_audit(audit.clone());

    let _ = audited.get("k1".to_string()).await.unwrap();
    let _ = audited.exists("k1".to_string()).await.unwrap();

    assert_eq!(audit.count_mod(), 0);
    assert_eq!(audit.count_del(), 0);
}

#[derive(Clone, Debug)]
struct TestValue {
    data: String,
}

impl ToAuditRecord for TestValue {
    fn to_audit_record(&self) -> String {
        self.data.clone()
    }
}

// In-memory repository with simple validation to force failures
struct InMemoryRepo(RwLock<HashMap<String, TestValue>>);

impl InMemoryRepo {
    fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    fn err(msg: &str) -> Status {
        Status::Timeout(msg.to_string())
    }
}

#[async_trait]
impl UpsertRepository<String, TestValue> for InMemoryRepo {
    type Error = Status;

    async fn upsert(&self, key: String, value: TestValue) -> Result<TestValue, Self::Error> {
        if key == "forbidden" {
            return Err(Self::err("forbidden key"));
        }
        let mut w = self.0.write().await;
        w.insert(key, value.clone());
        Ok(value)
    }

    async fn exists(&self, key: String) -> Result<bool, Self::Error> {
        let r = self.0.read().await;
        Ok(r.contains_key(&key))
    }
}

#[async_trait]
impl ReadOnlyRepository<String, TestValue> for InMemoryRepo {
    type ReadError = Status;

    async fn get(&self, key: String) -> Result<TestValue, Self::ReadError> {
        let r = self.0.read().await;
        r.get(&key).cloned().ok_or_else(|| Self::err("not found"))
    }
}

#[async_trait]
impl CanDelete<String, TestValue> for InMemoryRepo {
    type DeleteError = Status;

    async fn delete(&self, key: String) -> Result<(), Self::DeleteError> {
        let mut w = self.0.write().await;
        if w.remove(&key).is_some() {
            Ok(())
        } else {
            Err(Self::err("not found"))
        }
    }
}

#[async_trait]
impl UpsertRepositoryWithDelete<String, TestValue> for InMemoryRepo {}

// Mock audit service using tokio Mutex for sync trait methods
#[derive(Default)]
struct MockAuditService {
    modification_events: AtomicUsize,
    deletion_events: AtomicUsize,
}

impl MockAuditService {
    fn count_mod(&self) -> usize {
        self.modification_events.load(Ordering::Relaxed)
    }
    fn count_del(&self) -> usize {
        self.deletion_events.load(Ordering::Relaxed)
    }
}

impl AuditService for MockAuditService {
    // COVERAGE: ignore since it's stubbed out
    #[cfg_attr(coverage, coverage(off))]
    fn record_authorization(&self, _event: AuthorizationAuditEvent) -> anyhow::Result<()> {
        unreachable!()
    }

    fn record_resource_deletion(&self, _event: ResourceDeleteAuditEvent) -> Result<(), anyhow::Error> {
        self.deletion_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn record_resource_modification(&self, _event: ResourceModificationAuditEvent) -> Result<(), anyhow::Error> {
        self.modification_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    // COVERAGE: ignore since it's stubbed out
    #[cfg_attr(coverage, coverage(off))]
    fn record_token_validation(&self, _event: TokenValidationEvent) -> anyhow::Result<()> {
        unreachable!()
    }
}

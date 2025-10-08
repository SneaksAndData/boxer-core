use crate::services::base::upsert_repository::{ReadOnlyRepositoryWithFactory, ValueFactory};
use anyhow::anyhow;
use async_trait::async_trait;

struct SuccessfulValueFactory;

#[async_trait]
impl ValueFactory<String, String> for SuccessfulValueFactory {
    type CreateError = anyhow::Error;

    async fn create(&self, key: &String) -> Result<String, Self::CreateError> {
        let f = async { Ok(format!("Value for key: {}", key)) };
        f.await
    }
}

struct FailedValueFactory;

#[async_trait]
impl ValueFactory<String, String> for FailedValueFactory {
    type CreateError = anyhow::Error;

    async fn create(&self, key: &String) -> Result<String, Self::CreateError> {
        let f = async { Err(anyhow!("Error for key: {}", key)) };
        f.await
    }
}
#[tokio::test]
async fn test_create_value_successfully() {
    // Arrange
    let repo = super::InMemoryRepository::<String, String>::new();
    let key = "test-key".to_string();
    let result = repo.get(key, &SuccessfulValueFactory).await.unwrap();
    assert_eq!(result, "Value for key: test-key".to_string());
}

#[tokio::test]
async fn test_create_value_failure() {
    // Arrange
    let repo = super::InMemoryRepository::<String, String>::new();
    let key = "test-key".to_string();
    let result = repo.get(key, &FailedValueFactory).await.unwrap_err();
    assert_eq!(result.to_string(), "Error for key: test-key".to_string());
}

use crate::services::base::upsert_repository::UpsertRepository;
use crate::services::base::upsert_repository::UpsertRepositoryWithDelete;
use cedar_policy::SchemaFragment;

#[allow(dead_code)]
/// Represents a repository for schemas
pub type SchemaRepository = dyn UpsertRepositoryWithDelete<
        String,
        SchemaFragment,
        ReadError = anyhow::Error,
        Error = anyhow::Error,
        DeleteError = anyhow::Error,
    >;

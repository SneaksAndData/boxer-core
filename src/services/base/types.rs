use crate::services::base::upsert_repository::UpsertRepository;
use cedar_policy::SchemaFragment;

#[allow(dead_code)]
/// Represents a repository for schemas
pub type SchemaRepository = dyn UpsertRepository<String, SchemaFragment, Error = anyhow::Error>;

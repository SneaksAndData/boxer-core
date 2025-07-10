use cedar_policy::SchemaFragment;
use crate::services::base::upsert_repository::UpsertRepository;

#[allow(dead_code)]
/// Represents a repository for schemas
pub type SchemaRepository = dyn UpsertRepository<String, SchemaFragment, Error=anyhow::Error>;

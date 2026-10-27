pub mod resource_discovery_document;

pub mod read_write;

pub mod resource_set_registration;
pub mod schema_bound_resource_set_registration;
#[cfg(test)]
mod tests;

use crate::services::legacy::repositories::lookup_trie::schema_bound_trie_repository::SchemaBoundedTrieRepositoryData;
use crate::services::validation_service::path_segment::PathSegment;

pub type ResourceReadOnlyRepository = SchemaBoundedTrieRepositoryData<PathSegment>;

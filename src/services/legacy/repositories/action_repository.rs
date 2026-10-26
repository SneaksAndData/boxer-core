pub mod action_discovery_document;
pub mod read_write;

mod action_route_registration;
mod schema_bound_action_route_registration;
#[cfg(test)]
mod tests;

use crate::services::legacy::repositories::lookup_trie::schema_bound_trie_repository::SchemaBoundedTrieRepositoryData;
use crate::services::validation_service::request_segment::RequestSegment;

pub type ActionReadOnlyRepository = SchemaBoundedTrieRepositoryData<RequestSegment>;

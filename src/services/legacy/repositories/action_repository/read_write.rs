use crate::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use crate::services::backends::kubernetes::kubernetes_resource_manager::GenericKubernetesResourceManager;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::base::upsert_repository::UpsertRepositoryWithDelete;
use crate::services::legacy::repositories::action_repository::action_discovery_document::ActionDiscoveryDocument;
use crate::services::legacy::repositories::action_repository::schema_bound_action_route_registration::SchemaBoundActionSetRegistration;

pub type ActionDataRepository = dyn UpsertRepositoryWithDelete<
        (String, String),
        SchemaBoundActionSetRegistration,
        DeleteError = Status,
        Error = Status,
        ReadError = Status,
    >;

impl UpsertRepositoryWithDelete<(String, String), SchemaBoundActionSetRegistration>
    for KubernetesRepository<ActionDiscoveryDocument, GenericKubernetesResourceManager<ActionDiscoveryDocument>>
{
}

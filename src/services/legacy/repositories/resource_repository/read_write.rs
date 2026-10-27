use crate::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use crate::services::backends::kubernetes::kubernetes_resource_manager::GenericKubernetesResourceManager;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::base::upsert_repository::UpsertRepositoryWithDelete;
use crate::services::legacy::repositories::resource_repository::resource_discovery_document::ResourceDiscoveryDocument;
use crate::services::legacy::repositories::resource_repository::schema_bound_resource_set_registration::SchemaBoundResourceSetRegistration;

pub type ResourceDiscoveryDocumentRepository = dyn UpsertRepositoryWithDelete<
        (String, String),
        SchemaBoundResourceSetRegistration,
        DeleteError = Status,
        Error = Status,
        ReadError = Status,
    >;

impl UpsertRepositoryWithDelete<(String, String), SchemaBoundResourceSetRegistration>
    for KubernetesRepository<ResourceDiscoveryDocument, GenericKubernetesResourceManager<ResourceDiscoveryDocument>>
{
}

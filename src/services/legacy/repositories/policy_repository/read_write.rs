use crate::services::backends::kubernetes::kubernetes_repository::KubernetesRepository;
use crate::services::backends::kubernetes::kubernetes_resource_manager::GenericKubernetesResourceManager;
use crate::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use crate::services::base::upsert_repository::UpsertRepositoryWithDelete;
use crate::services::legacy::repositories::policy_repository::policy_document::PolicyDocument;
use crate::services::legacy::repositories::policy_repository::schema_bound_policy_set_registration::SchemaBoundPolicySetRegistration;

pub type PolicyDataRepository = dyn UpsertRepositoryWithDelete<
        (String, String),
        SchemaBoundPolicySetRegistration,
        DeleteError = Status,
        Error = Status,
        ReadError = Status,
    >;

impl UpsertRepositoryWithDelete<(String, String), SchemaBoundPolicySetRegistration>
    for KubernetesRepository<PolicyDocument, GenericKubernetesResourceManager<PolicyDocument>>
{
}

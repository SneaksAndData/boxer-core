use crate::services::audit::events::authorization_audit_event::AuthorizationAuditEvent;
use crate::services::audit::AuditService;
use crate::services::base::upsert_repository::ReadOnlyRepository;
use crate::services::observability::open_telemetry::metrics::authorization_metric::AuthorizationMetric;
use crate::services::observability::open_telemetry::metrics::metric_recorders::token_accepted::TokenAccepted;
use crate::services::observability::open_telemetry::metrics::metric_recorders::token_rejected::TokenRejected;
use crate::services::observability::open_telemetry::metrics::provider::MetricsProvider;
use crate::services::observability::open_telemetry::tracing::start_trace;
use crate::services::service_provider::ServiceProvider;
use crate::services::validation_service::path_segment::PathSegment;
use crate::services::validation_service::request_context::RequestContext;
use crate::services::validation_service::request_segment::RequestSegment;
use crate::services::validation_service::required_claims::RequiredClaims;
use crate::services::validation_service::schema_provider::SchemaProvider;
use crate::services::validation_service::ValidationService;
use async_trait::async_trait;
use cedar_policy::{Authorizer, Context, Entities, EntityUid, PolicySet, Request};
use log::{debug, info};
use opentelemetry::context::FutureExt;
use std::sync::Arc;

/// Abstracts the ReadOnlyRepository that contains EntityUid objects.
type EntityUidRepository<Key> = dyn ReadOnlyRepository<Key, EntityUid, ReadError = anyhow::Error>;

/// The repository that contains Action UIDs
pub type ActionRepository = EntityUidRepository<(String, Vec<RequestSegment>)>;

/// The repository that contains Resource UIDs
pub type ResourceRepository = EntityUidRepository<(String, Vec<PathSegment>)>;

/// Abstracts the repository that contains PolicySet objects.
pub type PolicyRepository = dyn ReadOnlyRepository<String, PolicySet, ReadError = anyhow::Error>;

pub struct CedarValidationService {
    authorizer: Authorizer,
    schema_provider: Arc<dyn SchemaProvider>,
    action_repository: Arc<ActionRepository>,
    resource_repository: Arc<ResourceRepository>,
    policy_repository: Arc<PolicyRepository>,
    audit: Arc<dyn AuditService>,
    metrics_provider: MetricsProvider,
}

impl CedarValidationService {
    /// Creates the new instance of the CedarValidationService
    pub fn new(
        schema_provider: Arc<dyn SchemaProvider>,
        action_repository: Arc<ActionRepository>,
        resource_repository: Arc<ResourceRepository>,
        policy_repository: Arc<PolicyRepository>,
        audit: Arc<dyn AuditService>,
        metrics_provider: MetricsProvider,
    ) -> Self {
        CedarValidationService {
            authorizer: Authorizer::new(),
            schema_provider,
            action_repository,
            resource_repository,
            policy_repository,
            audit,
            metrics_provider,
        }
    }
}

#[async_trait]
impl<Claims> ValidationService<Claims> for CedarValidationService
where
    Claims: RequiredClaims + Send + Sync + 'static,
{
    async fn validate(&self, claims: Claims, request_context: RequestContext) -> Result<(), anyhow::Error> {
        let ctx = start_trace("request_validation", None);
        let schema = self
            .schema_provider
            .get_schema(claims.get_validator_schema_id().clone())
            .with_context(ctx.clone())
            .await?;
        debug!("Cedar validation schemas: {:?}", schema);

        let action = self
            .action_repository
            .get((
                claims.get_validator_schema_id().clone(),
                request_context.clone().try_into()?,
            ))
            .with_context(ctx.clone())
            .await?;

        let resource = self
            .resource_repository
            .get((
                claims.get_validator_schema_id().clone(),
                request_context.clone().try_into()?,
            ))
            .with_context(ctx.clone())
            .await?;

        let policy_set = self
            .policy_repository
            .get(claims.get_validator_schema_id().clone())
            .with_context(ctx.clone())
            .await?;

        let actor: EntityUid = claims.get_principal().uid();

        let entities = Entities::empty().add_entities(vec![claims.get_principal().clone()], None)?;
        let request = Request::new(actor.clone(), action.clone(), resource.clone(), Context::empty(), None)?;
        let answer = self.authorizer.is_authorized(&request, &policy_set, &entities);

        info!(
            "validation {:?} for actor {:?} action {:?} on resource {:?}",
            answer,
            actor.to_string(),
            action.to_string(),
            resource.to_string()
        );

        self.audit
            .record_authorization(AuthorizationAuditEvent::new(&actor, &action, &resource, &answer))?;

        match answer.decision() {
            cedar_policy::Decision::Allow => {
                let metric: TokenAccepted = self.metrics_provider.get();
                metric.increment(actor, action, resource);
                Ok(())
            }
            cedar_policy::Decision::Deny => {
                let metric: TokenRejected = self.metrics_provider.get();
                metric.increment(actor, action, resource);
                anyhow::bail!("Access denied")
            }
        }
    }
}

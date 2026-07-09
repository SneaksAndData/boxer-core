use crate::contracts::internal_token::v1::token::InternalToken;
use crate::services::observability::open_telemetry::metrics::metric_recorders::token_issued::{
    TokenIssued, TokenIssuedMetric,
};
use crate::services::observability::open_telemetry::metrics::metric_recorders::token_lifetime::{
    TokenLifetime, TokenLifetimeMetric,
};
use crate::services::observability::open_telemetry::metrics::provider::MetricsProvider;
use crate::services::service_provider::ServiceProvider;
use crate::services::token_provider::TokenProvider;
use crate::services::token_provider::external_identity::ExternalIdentity;
use crate::services::token_provider::principal_service::PrincipalService;
use async_trait::async_trait;
use josekit::jwe::{Dir, JweHeader};
use josekit::jwt;
use josekit::jwt::JwtPayload;
use std::sync::Arc;
use std::time::Duration;

pub struct EncryptedTokenService {
    principal_service: Arc<dyn PrincipalService>,
    encrypt_secret: Arc<Vec<u8>>,
    audience: String,
    key_id: String,
    issuer: String,
    content_encryption: String,
    token_duration: Duration,
    metrics_provider: MetricsProvider,
}

#[async_trait]
impl TokenProvider for EncryptedTokenService {
    async fn issue_token(&self, identity: ExternalIdentity) -> Result<String, anyhow::Error> {
        let principal = self.principal_service.get_principal(identity.clone()).await?;
        let schema_name = principal.get_schema_id().clone();
        let schemas = self.principal_service.get_schemas(schema_name.clone()).await?;
        let validator_schema_id = self.principal_service.get_validator_schema(identity.clone()).await?;
        let payload: JwtPayload = InternalToken::new(
            principal.get_entity().clone(),
            schemas,
            identity.user_id.clone(),
            identity.identity_provider.clone(),
            schema_name,
            self.token_duration.clone(),
            validator_schema_id,
        )
        .try_into()?;

        let mut header = JweHeader::new();
        header.set_token_type("JWT");
        header.set_audience(vec![self.audience.as_str()]);
        header.set_issuer(self.issuer.clone());
        header.set_content_encryption(&self.content_encryption);
        header.set_key_id(&self.key_id);

        let encrypter = Dir.encrypter_from_bytes(&*self.encrypt_secret)?;
        let token = jwt::encode_with_encrypter(&payload, &header, &encrypter).map_err(|e| anyhow::anyhow!(e))?;

        let ti: TokenIssued = self.metrics_provider.get();
        ti.increment(identity.identity_provider.clone(), identity.user_id.clone());

        let tl: TokenLifetime = self.metrics_provider.get();
        tl.increment(identity.identity_provider, identity.user_id, self.token_duration);

        Ok(token)
    }
}

impl EncryptedTokenService {
    pub fn new(
        principal_service: Arc<dyn PrincipalService>,
        encrypt_secret: Arc<Vec<u8>>,
        key_id: String,
        audience: String,
        issuer: String,
        content_encryption: String,
        metrics_provider: MetricsProvider,
    ) -> Self {
        EncryptedTokenService {
            principal_service,
            encrypt_secret,
            key_id,
            audience,
            issuer,
            content_encryption,
            token_duration: Duration::from_secs(3600),
            metrics_provider,
        }
    }
}

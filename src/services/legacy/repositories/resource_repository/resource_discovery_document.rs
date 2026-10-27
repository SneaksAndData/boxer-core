use crate::services::backends::kubernetes::kubernetes_repository::soft_delete_resource::SoftDeleteResource;
use crate::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use crate::services::legacy::repositories::lookup_trie::{EntityCollectionResource, SchemaBoundResource};
use crate::services::legacy::repositories::resource_repository::resource_set_registration::ResourceRouteRegistration;
use crate::services::legacy::repositories::resource_repository::schema_bound_resource_set_registration::SchemaBoundResourceSetRegistration;
use crate::services::validation_service::path_segment::PathSegment;
use crate::services::validation_service::path_segment::PathSegment::{Parameter, Static};
use cedar_policy::EntityUid;
use futures::Stream;
use futures::StreamExt;
use futures_util::stream;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "ResourceDiscoveryDocument",
    plural = "resource-discovery-documents",
    singular = "resource-discovery-document",
    namespaced
)]

pub struct ResourceDiscoveryDocumentSpec {
    pub active: bool,
    pub hostname: String,
    pub routes: Vec<ResourceRoute>,
    pub schema: String,
}

impl EntityCollectionResource<PathSegment> for ResourceDiscoveryDocument {
    fn stream(self) -> impl Stream<Item=Result<(Vec<PathSegment>, EntityUid, bool), anyhow::Error>> + Send + Sync {
        let active = self.spec.active;
        stream::iter(self.spec.routes)
            .zip(stream::repeat(active))
            .map(move |(route, active)| {
                let action_uid: EntityUid = EntityUid::from_str(&route.resource_uid).map_err(anyhow::Error::from)?;
                let mut key: Vec<PathSegment> = vec![];
                let segments: Vec<PathSegment> = route.try_into()?;
                key.extend(segments);
                Ok((key, action_uid, active))
            })
    }
}

impl Default for ResourceDiscoveryDocument {
    fn default() -> Self {
        ResourceDiscoveryDocument {
            metadata: ObjectMeta::default(),
            spec: ResourceDiscoveryDocumentSpec::default(),
        }
    }
}

impl From<&SchemaBoundResourceSetRegistration> for ResourceDiscoveryDocumentSpec {
    fn from(value: &SchemaBoundResourceSetRegistration) -> Self {
        let mut routes = Vec::<ResourceRoute>::new();

        for route in &value.routes {
            let action_route = ResourceRoute {
                route_template: route.route_template.clone(),
                resource_uid: route.resource_uid.to_string(),
            };
            routes.push(action_route)
        }
        ResourceDiscoveryDocumentSpec {
            active: true,
            hostname: value.hostname.clone(),
            routes,
            schema: value.schema.clone(),
        }
    }
}

impl Into<SchemaBoundResourceSetRegistration> for ResourceDiscoveryDocumentSpec {
    fn into(self) -> SchemaBoundResourceSetRegistration {
        let routes: Vec<ResourceRouteRegistration> = self
            .routes
            .into_iter()
            .map(|route| ResourceRouteRegistration {
                route_template: route.route_template,
                resource_uid: route.resource_uid,
            })
            .collect();

        SchemaBoundResourceSetRegistration {
            hostname: self.hostname,
            routes,
            schema: self.schema.clone(),
        }
    }
}

impl SoftDeleteResource for ResourceDiscoveryDocument {
    fn is_deleted(&self) -> bool {
        !self.spec.active
    }

    fn set_deleted(&mut self) {
        self.spec.active = false;
    }

    fn clear_managed_fields(&mut self) {
        self.metadata.managed_fields = None;
    }
}

impl UpdateLabels for ResourceDiscoveryDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRoute {
    route_template: String,
    resource_uid: String,
}

impl TryInto<Vec<PathSegment>> for ResourceRoute {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<PathSegment>, Self::Error> {
        let mut segments: Vec<PathSegment> = vec![];
        for segment in self.route_template.split('/') {
            if segment.is_empty() {
                continue;
            }
            if segment.starts_with('{') && segment.ends_with('}') {
                segments.push(Parameter)
            } else {
                segments.push(Static(segment.to_string()))
            }
        }
        Ok(segments)
    }
}

impl SchemaBoundResource for ResourceDiscoveryDocument {
    fn schema(&self) -> String {
        self.spec.schema.clone()
    }
}

use crate::services::backends::kubernetes::kubernetes_repository::schema_repository::schema_document::{
    SchemaDocument, SchemaDocumentSpec,
};
use crate::services::backends::kubernetes::kubernetes_resource_manager::object_owner_mark::ObjectOwnerMark;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use maplit::btreemap;

#[test]
fn test_object_owner_mark() {
    let resource = SchemaDocument {
        metadata: ObjectMeta {
            labels: Some(btreemap! {"boxer.io".to_string() => "owner".to_string()}),
            name: Some("name".to_string()),
            namespace: Some("namespace".to_string()),
            ..Default::default()
        },
        spec: SchemaDocumentSpec::default(),
    };

    let owner_mark = ObjectOwnerMark::new("boxer.io", "owner");

    assert!(owner_mark.is_owned(&resource));
    assert_eq!(owner_mark.get_resource_owner(&resource), Some("owner".to_string()));
}

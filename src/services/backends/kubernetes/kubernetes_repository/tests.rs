use crate::services::backends::kubernetes::kubernetes_repository::TryIntoObjectRef;
use crate::services::backends::kubernetes::kubernetes_repository::schema_repository::schema_document::SchemaDocument;
use kube::runtime::reflector::ObjectRef;

#[test]
fn test_to_object_ref() {
    let or: ObjectRef<SchemaDocument> = "~~~test-object!!default"
        .to_string()
        .try_into_object_ref("default".to_string())
        .unwrap();

    assert_eq!(or.name, "test-object--default");
    assert_eq!(or.namespace, Some("default".to_string()));
}

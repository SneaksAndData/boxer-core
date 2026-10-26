use crate::services::legacy::prefix_tree::MutablePrefixTree;
use crate::services::legacy::prefix_tree::PrefixTree;
use crate::services::legacy::prefix_tree::naive_tree::NaiveTrie;
use crate::services::legacy::prefix_tree::parametrized_matcher::ParametrizedMatcher;
use crate::services::legacy::prefix_tree::trie_bucket::hash_bucket::HashTrieBucket;
use crate::services::legacy::prefix_tree::trie_bucket::request_segment_bucket::PrioritizedBucket;
use crate::services::validation_service::http_method::HTTPMethod;
use crate::services::validation_service::path_segment::PathSegment;
use crate::services::validation_service::request_context::RequestContext;
use crate::services::validation_service::request_segment::RequestSegment;
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test_case("api/v1/resources/resource1" => matches Some("value1"); "simple insert")]
#[test_case("" => matches None; "insert empty key")]
#[tokio::test]
async fn test_insert(key: &str) -> Option<&'static str> {
    let mut trie = NaiveTrie::<HashTrieBucket<u8, &str>>::new();
    NaiveTrie::insert(&mut trie, key, "value1").await;
    trie.get(key).await
}

#[test_case(("api/v1/resources/resource1", "api/v1/resources/resource1") => matches Some("value1"); "exact match"
)]
#[test_case(("api/v1/resources/resource1", "api/v1/resources/resource") => matches None; "partial match"
)]
#[test_case(("", "api/v1/resources/") => matches None; "empty key with query")]
#[test_case(("", "") => matches None; "empty key with empty query")]
#[tokio::test]
async fn test_partial_query(keys: (&str, &str)) -> Option<&'static str> {
    let (key, query) = keys;
    let mut trie = NaiveTrie::<HashTrieBucket<u8, &str>>::new();
    trie.insert(key, "value1").await;
    trie.get(query).await
}

#[tokio::test]
async fn test_overwrite_existing_value() {
    let key = "api/v1/resources/resource1";
    let mut trie = NaiveTrie::<HashTrieBucket<u8, &str>>::new();
    trie.insert(key, "value1").await;
    trie.insert(key, "value2").await;
    trie.insert(key, "value3").await;
    let value = trie.get(key).await.expect("Expected to find the key in the trie");
    assert_eq!(value, "value3");
}

#[tokio::test]
async fn test_delete_existing_value() {
    let key = "api/v1/resources/resource1";
    let mut trie = NaiveTrie::<HashTrieBucket<u8, &str>>::new();
    trie.insert(key, "value1").await;
    trie.get(key).await.expect("Expected to find the key in the trie");

    trie.delete(key).await.expect("Expected to find the key in the trie");
    let value = trie.get(key).await;
    assert!(value.is_none(), "Expected the key to be deleted from the trie");
}

#[tokio::test]
async fn test_delete_nonexistent() {
    let key = "api/v1/resources/resource1";
    let trie = NaiveTrie::<HashTrieBucket<u8, &str>>::new();

    trie.delete(key).await;
    let value = trie.get(key).await;
    assert!(
        value.is_none(),
        "Expected the key to be deleted from the trie, but was: {:?}",
        value
    );
}

#[tokio::test]
async fn get_nonexistent_branch() {
    let key = "api/v1/resources/resource1";
    let mut trie = NaiveTrie::<HashTrieBucket<u8, u32>>::new();
    trie.insert(key, 10).await;

    let key = "api/v1/resources/resource2";
    assert_eq!(trie.get(&key).await, None);
}

#[tokio::test]
async fn branching_paths() {
    let mut trie: NaiveTrie<HashTrieBucket<String, i32>> = NaiveTrie::new();
    let p1 = vec!["root".to_string(), "left".to_string()];
    let p2 = vec!["root".to_string(), "right".to_string()];
    trie.insert(&p1, 1).await;
    trie.insert(&p2, 2).await;

    assert_eq!(trie.get(&p1).await, Some(1));
    assert_eq!(trie.get(&p2).await, Some(2));
    assert_eq!(
        trie.get(&vec!["root".to_string()]).await,
        None,
        "No value at intermediate node"
    );
}

fn wrapped_pretty_assert(expected: String) -> impl Fn(String) {
    move |actual: String| assert_eq!(actual, expected)
}

#[test_case("www.example.com/api/v1/resources/resource/resource1" => using wrapped_pretty_assert("value0".to_string()); "with parameter match"
)]
#[test_case("www.example.com/api/v1/resources/my-resource" => using wrapped_pretty_assert("value1".to_string()); "with exact match"
)]
#[test_case("www.example.com/api/v1/resources/my-idy-resource/ids/an-id" => using wrapped_pretty_assert("value2".to_string()); "with deeper parameter in path"
)]
#[test_case("www.example.com/i-do-not/know/what/i-am/doing" => using wrapped_pretty_assert("value3".to_string()); "with full parameter path"
)]
#[test_case("www.example.com/api/v1/resources/my-resource/ids/my-id" => using wrapped_pretty_assert("value4".to_string()); "with full exact path in the end"
)]
#[tokio::test]
async fn test_path_segment_matchers(key: &str) -> String {
    let segments = vec![
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Static("api".to_string())),
            RequestSegment::Path(PathSegment::Static("v1".to_string())),
            RequestSegment::Path(PathSegment::Static("resources".to_string())),
            RequestSegment::Path(PathSegment::Static("resource".to_string())),
            RequestSegment::Path(PathSegment::Parameter),
        ],
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Static("api".to_string())),
            RequestSegment::Path(PathSegment::Static("v1".to_string())),
            RequestSegment::Path(PathSegment::Static("resources".to_string())),
            RequestSegment::Path(PathSegment::Parameter),
        ],
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Static("api".to_string())),
            RequestSegment::Path(PathSegment::Static("v1".to_string())),
            RequestSegment::Path(PathSegment::Static("resources".to_string())),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Static("ids".to_string())),
            RequestSegment::Path(PathSegment::Parameter),
        ],
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Parameter),
            RequestSegment::Path(PathSegment::Parameter),
        ],
        vec![
            RequestSegment::Hostname("www.example.com".to_string()),
            RequestSegment::Verb(HTTPMethod::Get),
            RequestSegment::Path(PathSegment::Static("api".to_string())),
            RequestSegment::Path(PathSegment::Static("v1".to_string())),
            RequestSegment::Path(PathSegment::Static("resources".to_string())),
            RequestSegment::Path(PathSegment::Static("my-resource".to_string())),
            RequestSegment::Path(PathSegment::Static("ids".to_string())),
            RequestSegment::Path(PathSegment::Static("my-id".to_string())),
        ],
    ];
    let mut trie = NaiveTrie::<PrioritizedBucket<RequestSegment, String>>::new();

    for i in 0..segments.len() {
        trie.insert(segments[i].clone(), format!("value{}", i)).await;
    }

    let rc: Vec<RequestSegment> = RequestContext::new(format!("http://{key}"), "GET".to_string())
        .try_into()
        .unwrap();

    println!("Request Context Segments: {:?}", rc);
    trie.get(&rc).await.unwrap_or("".to_string())
}

impl ParametrizedMatcher for u8 {
    fn is_parameter(&self) -> bool {
        false
    }
}

impl ParametrizedMatcher for String {
    fn is_parameter(&self) -> bool {
        false
    }
}

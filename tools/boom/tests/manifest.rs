use boom::manifest::{parse_manifests, priority_tier, sort_by_priority, ResourceDescriptor};

#[test]
fn parse_single_doc() {
    let yaml = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web
  namespace: production
"#;
    let resources = parse_manifests(yaml);
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].api_version, "apps/v1");
    assert_eq!(resources[0].kind, "Deployment");
    assert_eq!(resources[0].name, "web");
    assert_eq!(resources[0].namespace, Some("production".to_string()));
}

#[test]
fn parse_multi_doc() {
    let yaml = r#"apiVersion: v1
kind: Namespace
metadata:
  name: test
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: config
---

---
apiVersion: v1
kind: Secret
metadata:
  name: creds
"#;
    let resources = parse_manifests(yaml);
    assert_eq!(resources.len(), 3, "blank section between separators should be skipped");
}

#[test]
fn tier_classification() {
    assert_eq!(priority_tier("Namespace"), 0);
    assert_eq!(priority_tier("CustomResourceDefinition"), 0);
    assert_eq!(priority_tier("Secret"), 1);
    assert_eq!(priority_tier("PersistentVolumeClaim"), 1);
    assert_eq!(priority_tier("Deployment"), 2);
}

#[test]
fn sort_priority() {
    let mut resources = vec![
        ResourceDescriptor {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            name: "web".to_string(),
            namespace: None,
            raw: serde_yaml::Value::Null,
        },
        ResourceDescriptor {
            api_version: "v1".to_string(),
            kind: "ConfigMap".to_string(),
            name: "config".to_string(),
            namespace: None,
            raw: serde_yaml::Value::Null,
        },
        ResourceDescriptor {
            api_version: "v1".to_string(),
            kind: "Namespace".to_string(),
            name: "ns".to_string(),
            namespace: None,
            raw: serde_yaml::Value::Null,
        },
        ResourceDescriptor {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            name: "api".to_string(),
            namespace: None,
            raw: serde_yaml::Value::Null,
        },
    ];

    sort_by_priority(&mut resources);

    assert_eq!(resources[0].kind, "Namespace", "tier 0 should come first");
    assert_eq!(resources[1].kind, "ConfigMap", "tier 1 should come second");
    assert_eq!(resources[2].kind, "Deployment", "tier 2 should come last");
    assert_eq!(resources[2].name, "web", "stable sort preserves relative order of Deployments");
    assert_eq!(resources[3].name, "api", "second Deployment keeps its position");
}

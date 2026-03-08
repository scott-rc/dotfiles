use boom::manifest;

#[test]
fn is_cluster_scoped_returns_true_for_cluster_scoped_kinds() {
    let cluster_scoped = [
        "ClusterRole",
        "ClusterRoleBinding",
        "Namespace",
        "CustomResourceDefinition",
        "PersistentVolume",
        "StorageClass",
        "IngressClass",
        "PriorityClass",
    ];
    for kind in cluster_scoped {
        assert!(
            manifest::is_cluster_scoped(kind),
            "{kind} should be cluster-scoped"
        );
    }
}

#[test]
fn is_cluster_scoped_returns_false_for_namespaced_kinds() {
    let namespaced = ["Deployment", "Service", "ConfigMap"];
    for kind in namespaced {
        assert!(
            !manifest::is_cluster_scoped(kind),
            "{kind} should not be cluster-scoped"
        );
    }
}

#[test]
fn filter_mixed_manifests_yields_only_cluster_scoped() {
    let yaml = r#"
apiVersion: v1
kind: Namespace
metadata:
  name: test-ns
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-deploy
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: my-role
---
apiVersion: v1
kind: Service
metadata:
  name: my-svc
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: my-cm
"#;

    let resources = manifest::parse_manifests(yaml);
    let cluster_only: Vec<_> = resources
        .iter()
        .filter(|r| manifest::is_cluster_scoped(&r.kind))
        .collect();

    assert_eq!(cluster_only.len(), 2);
    assert_eq!(cluster_only[0].kind, "Namespace");
    assert_eq!(cluster_only[1].kind, "ClusterRole");
}

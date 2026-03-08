#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
    pub raw: serde_yaml::Value,
}

fn extract_field(doc: &serde_yaml::Value, key: &str) -> String {
    doc.get(key)
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub fn parse_manifests(yaml: &str) -> Vec<ResourceDescriptor> {
    yaml.split("\n---")
        .filter(|section| !section.trim().is_empty())
        .filter_map(|section| {
            let doc: serde_yaml::Value = serde_yaml::from_str(section).ok()?;
            let metadata = doc.get("metadata")?;
            let namespace = metadata
                .get("namespace")
                .and_then(serde_yaml::Value::as_str)
                .map(String::from);
            Some(ResourceDescriptor {
                api_version: extract_field(&doc, "apiVersion"),
                kind: extract_field(&doc, "kind"),
                name: extract_field(metadata, "name"),
                namespace,
                raw: doc,
            })
        })
        .collect()
}

pub fn is_cluster_scoped(kind: &str) -> bool {
    matches!(
        kind,
        "Namespace"
            | "ClusterRole"
            | "ClusterRoleBinding"
            | "CustomResourceDefinition"
            | "PersistentVolume"
            | "StorageClass"
            | "IngressClass"
            | "PriorityClass"
    )
}

pub fn priority_tier(kind: &str) -> u8 {
    match kind {
        "Namespace" | "ServiceAccount" | "CustomResourceDefinition" => 0,
        "ConfigMap" | "Secret" | "PersistentVolumeClaim" | "StorageClass" => 1,
        _ => 2,
    }
}

pub fn sort_by_priority(resources: &mut [ResourceDescriptor]) {
    resources.sort_by_key(|r| priority_tier(&r.kind));
}

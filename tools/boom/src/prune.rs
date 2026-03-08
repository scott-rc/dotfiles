use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub name: String,
    pub kind: String,
    pub namespace: String,
}

pub fn identify_stale(
    deployed: &[ResourceDescriptor],
    existing: &[ResourceDescriptor],
) -> Vec<ResourceDescriptor> {
    let deployed_set: HashSet<(&str, &str)> = deployed
        .iter()
        .map(|r| (r.name.as_str(), r.kind.as_str()))
        .collect();

    existing
        .iter()
        .filter(|r| !deployed_set.contains(&(r.name.as_str(), r.kind.as_str())))
        .cloned()
        .collect()
}

pub async fn execute(client: &kube::Client, stale: &[ResourceDescriptor]) -> Result<(), String> {
    use kube::api::{Api, ApiResource, DeleteParams, DynamicObject, PropagationPolicy};

    for resource in stale {
        let ar = ApiResource {
            group: String::new(),
            version: "v1".to_string(),
            api_version: "v1".to_string(),
            kind: resource.kind.clone(),
            plural: crate::deploy::pluralize(&resource.kind),
        };

        let api: Api<DynamicObject> =
            Api::namespaced_with(client.clone(), &resource.namespace, &ar);

        let dp = DeleteParams {
            propagation_policy: Some(PropagationPolicy::Background),
            ..Default::default()
        };

        api.delete(&resource.name, &dp)
            .await
            .map_err(|e| format!("failed to delete {}/{}: {e}", resource.kind, resource.name))?;
    }

    Ok(())
}

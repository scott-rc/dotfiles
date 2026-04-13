use std::time::Instant;

use kube::Api;
use kube::api::{ApiResource, DynamicObject, Patch, PatchParams};
use tokio::task::JoinSet;

use crate::manifest::{self, ResourceDescriptor};
use crate::monitor::{self, ResourceState};
use crate::output;

pub async fn run(
    client: kube::Client,
    namespace: &str,
    mut resources: Vec<ResourceDescriptor>,
    verify_result: bool,
    global_timeout: u64,
) -> bool {
    manifest::sort_by_priority(&mut resources);

    let start = Instant::now();
    let mut all_ok = true;
    let mut summary_rows: Vec<(String, String, String, String)> = Vec::new();

    for tier in 0..=2u8 {
        let tier_resources: Vec<&ResourceDescriptor> = resources
            .iter()
            .filter(|r| manifest::priority_tier(&r.kind) == tier)
            .collect();

        if tier_resources.is_empty() {
            continue;
        }

        let mut set = JoinSet::new();

        for resource in tier_resources {
            let client = client.clone();
            let kind = resource.kind.clone();
            let name = resource.name.clone();
            let api_version = resource.api_version.clone();
            let raw = resource.raw.clone();
            let ns = resource.namespace.clone().unwrap_or(namespace.to_string());
            let resource_start = Instant::now();

            set.spawn(async move {
                let result = apply_resource(&client, &api_version, &kind, &name, &ns, &raw).await;
                let elapsed = resource_start.elapsed();
                (kind, name, result, elapsed)
            });
        }

        while let Some(join_result) = set.join_next().await {
            match join_result {
                Ok((kind, name, Ok(()), elapsed)) => {
                    output::success(&format!("[OK] {kind}/{name}"));
                    summary_rows.push((
                        name,
                        kind,
                        "OK".to_string(),
                        format!("{:.1}s", elapsed.as_secs_f64()),
                    ));
                }
                Ok((kind, name, Err(err), elapsed)) => {
                    output::error(&format!("[FAIL] {kind}/{name}: {err}"));
                    summary_rows.push((
                        name,
                        kind,
                        "Failed".to_string(),
                        format!("{:.1}s", elapsed.as_secs_f64()),
                    ));
                    all_ok = false;
                }
                Err(err) => {
                    output::error(&format!("[FAIL] join error: {err}"));
                    all_ok = false;
                }
            }
        }
    }

    if !all_ok {
        output::summary_table(&summary_rows);
        return false;
    }

    if !verify_result {
        output::summary_table(&summary_rows);
        return true;
    }

    let state = monitor::watch_resources(&client, &resources, global_timeout).await;
    let total_elapsed = start.elapsed();
    match state {
        ResourceState::Ready => {
            for row in &mut summary_rows {
                row.3 = format!("{:.1}s", total_elapsed.as_secs_f64());
            }
            output::summary_table(&summary_rows);
            true
        }
        ResourceState::Failed => {
            for resource in &resources {
                let diag = monitor::collect_diagnostics(&client, resource).await;
                if !diag.is_empty() {
                    output::error(&diag);
                }
            }
            output::summary_table(&summary_rows);
            std::process::exit(1);
        }
        ResourceState::NotReady => {
            output::warn("[boom] timed out waiting for resources to become ready");
            for resource in &resources {
                let diag = monitor::collect_diagnostics(&client, resource).await;
                if !diag.is_empty() {
                    output::error(&diag);
                }
            }
            output::summary_table(&summary_rows);
            std::process::exit(70);
        }
    }
}

async fn apply_resource(
    client: &kube::Client,
    api_version: &str,
    kind: &str,
    name: &str,
    namespace: &str,
    raw: &serde_yaml::Value,
) -> Result<(), String> {
    let (group, version) = parse_api_version(api_version);

    let ar = ApiResource {
        group: group.to_string(),
        version: version.to_string(),
        api_version: api_version.to_string(),
        kind: kind.to_string(),
        plural: pluralize(kind),
    };

    let api: Api<DynamicObject> =
        if kind == "Namespace" || kind == "CustomResourceDefinition" || kind == "StorageClass" {
            Api::all_with(client.clone(), &ar)
        } else {
            Api::namespaced_with(client.clone(), namespace, &ar)
        };

    let json_value =
        serde_json::to_value(raw).map_err(|e| format!("failed to convert to JSON: {e}"))?;

    let params = PatchParams::apply("boom");
    api.patch(name, &params, &Patch::Apply(json_value))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn parse_api_version(api_version: &str) -> (&str, &str) {
    match api_version.rsplit_once('/') {
        Some((group, version)) => (group, version),
        None => ("", api_version),
    }
}

pub fn pluralize(kind: &str) -> String {
    let lower = kind.to_lowercase();
    if lower.ends_with('s') {
        format!("{lower}es")
    } else if lower.ends_with("cy") {
        format!("{}ies", &lower[..lower.len() - 1])
    } else {
        format!("{lower}s")
    }
}

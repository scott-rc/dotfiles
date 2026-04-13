use std::process;

use kube::Api;
use kube::api::{ApiResource, DynamicObject, Patch, PatchParams};
use serde_json::json;

use crate::manifest::ResourceDescriptor;
use crate::monitor;

pub fn build_restart_patch(timestamp: &str) -> serde_json::Value {
    json!({
        "spec": {
            "template": {
                "metadata": {
                    "annotations": {
                        "kubectl.kubernetes.io/restartedAt": timestamp
                    }
                }
            }
        }
    })
}

pub async fn run(
    client: kube::Client,
    namespace: &str,
    deployments: &[String],
    statefulsets: &[String],
    daemonsets: &[String],
    verify_result: bool,
    global_timeout: u64,
) {
    let timestamp = chrono_now();
    let patch = build_restart_patch(&timestamp);

    let kinds = [
        ("Deployment", "apps/v1", deployments),
        ("StatefulSet", "apps/v1", statefulsets),
        ("DaemonSet", "apps/v1", daemonsets),
    ];

    let mut patched_resources = Vec::new();

    for (kind, api_version, names) in &kinds {
        for name in *names {
            patch_resource(&client, namespace, kind, api_version, name, &patch).await;
            patched_resources.push(ResourceDescriptor {
                api_version: (*api_version).to_string(),
                kind: (*kind).to_string(),
                name: name.clone(),
                namespace: Some(namespace.to_string()),
                raw: serde_yaml::Value::Null,
            });
        }
    }

    if verify_result && !patched_resources.is_empty() {
        let state = monitor::watch_resources(&client, &patched_resources, global_timeout).await;
        match state {
            monitor::ResourceState::Ready => {
                eprintln!("[boom] all restarts complete");
            }
            monitor::ResourceState::Failed => {
                for resource in &patched_resources {
                    let diag = monitor::collect_diagnostics(&client, resource).await;
                    if !diag.is_empty() {
                        eprintln!("{diag}");
                    }
                }
                process::exit(1);
            }
            monitor::ResourceState::NotReady => {
                eprintln!("[boom] timed out waiting for restarts to complete");
                for resource in &patched_resources {
                    let diag = monitor::collect_diagnostics(&client, resource).await;
                    if !diag.is_empty() {
                        eprintln!("{diag}");
                    }
                }
                process::exit(70);
            }
        }
    }
}

async fn patch_resource(
    client: &kube::Client,
    namespace: &str,
    kind: &str,
    api_version: &str,
    name: &str,
    patch: &serde_json::Value,
) {
    let (group, version) = crate::deploy::parse_api_version(api_version);
    let ar = ApiResource {
        group: group.to_string(),
        version: version.to_string(),
        api_version: api_version.to_string(),
        kind: kind.to_string(),
        plural: crate::deploy::pluralize(kind),
    };

    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);
    let params = PatchParams::apply("boom");
    match api.patch(name, &params, &Patch::Merge(patch.clone())).await {
        Ok(_) => eprintln!("[boom] patched {kind}/{name}"),
        Err(e) => {
            eprintln!("boom: failed to patch {kind}/{name}: {e}");
            process::exit(1);
        }
    }
}

fn chrono_now() -> String {
    let output = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("boom: failed to get current time: {e}");
            process::exit(1);
        });
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

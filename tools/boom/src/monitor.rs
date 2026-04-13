use std::collections::HashMap;
use std::fmt::{self, Write};
use std::time::Duration;

use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use kube::api::{ApiResource, DynamicObject, ListParams, LogParams};

use crate::manifest::ResourceDescriptor;

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceState {
    Ready,
    NotReady,
    Failed,
}

impl fmt::Display for ResourceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ready => write!(f, "Ready"),
            Self::NotReady => write!(f, "NotReady"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

pub fn is_ready(kind: &str, resource: &serde_json::Value) -> ResourceState {
    match kind {
        "Deployment" => check_deployment(resource),
        "StatefulSet" => check_statefulset(resource),
        "DaemonSet" => check_daemonset(resource),
        "Pod" => check_pod(resource),
        "Job" => check_job(resource),
        "ConfigMap" | "Secret" | "PersistentVolumeClaim" | "Service" => ResourceState::Ready,
        _ => ResourceState::NotReady,
    }
}

fn int_field(val: &serde_json::Value, path: &[&str]) -> i64 {
    let mut v = val;
    for key in path {
        v = match v.get(key) {
            Some(inner) => inner,
            None => return 0,
        };
    }
    v.as_i64().unwrap_or(0)
}

fn check_deployment(resource: &serde_json::Value) -> ResourceState {
    let desired = int_field(resource, &["spec", "replicas"]);
    let available = int_field(resource, &["status", "availableReplicas"]);
    let updated = int_field(resource, &["status", "updatedReplicas"]);
    if available >= desired && updated >= desired {
        ResourceState::Ready
    } else {
        ResourceState::NotReady
    }
}

fn check_statefulset(resource: &serde_json::Value) -> ResourceState {
    let desired = int_field(resource, &["spec", "replicas"]);
    let ready = int_field(resource, &["status", "readyReplicas"]);
    if ready >= desired {
        ResourceState::Ready
    } else {
        ResourceState::NotReady
    }
}

fn check_daemonset(resource: &serde_json::Value) -> ResourceState {
    let desired = int_field(resource, &["status", "desiredNumberScheduled"]);
    let ready = int_field(resource, &["status", "numberReady"]);
    if ready >= desired {
        ResourceState::Ready
    } else {
        ResourceState::NotReady
    }
}

fn check_pod(resource: &serde_json::Value) -> ResourceState {
    // Check for terminal failure states first
    if let Some(statuses) = resource
        .get("status")
        .and_then(|s| s.get("containerStatuses"))
        .and_then(|c| c.as_array())
    {
        for cs in statuses {
            let reason = cs
                .get("state")
                .and_then(|s| s.get("waiting"))
                .and_then(|w| w.get("reason"))
                .and_then(|r| r.as_str())
                .unwrap_or("");
            if reason == "CrashLoopBackOff" || reason == "ImagePullBackOff" {
                return ResourceState::Failed;
            }
        }
    }

    let phase = resource
        .get("status")
        .and_then(|s| s.get("phase"))
        .and_then(|p| p.as_str())
        .unwrap_or("");

    if phase == "Succeeded" {
        return ResourceState::Ready;
    }

    if phase == "Running" {
        let all_ready = resource
            .get("status")
            .and_then(|s| s.get("containerStatuses"))
            .and_then(|c| c.as_array())
            .is_some_and(|statuses| {
                !statuses.is_empty()
                    && statuses.iter().all(|cs| {
                        cs.get("ready")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false)
                    })
            });
        if all_ready {
            return ResourceState::Ready;
        }
    }

    ResourceState::NotReady
}

fn check_job(resource: &serde_json::Value) -> ResourceState {
    let failed = int_field(resource, &["status", "failed"]);
    let backoff_limit = int_field(resource, &["spec", "backoffLimit"]);
    if failed > backoff_limit {
        return ResourceState::Failed;
    }

    let succeeded = int_field(resource, &["status", "succeeded"]);
    let completions = int_field(resource, &["spec", "completions"]);
    if succeeded >= completions {
        ResourceState::Ready
    } else {
        ResourceState::NotReady
    }
}

async fn check_resource(client: &kube::Client, resource: &ResourceDescriptor) -> ResourceState {
    let ns = resource.namespace.as_deref().unwrap_or("default");
    let (group, version) = crate::deploy::parse_api_version(&resource.api_version);

    let ar = ApiResource {
        group: group.to_string(),
        version: version.to_string(),
        api_version: resource.api_version.clone(),
        kind: resource.kind.clone(),
        plural: crate::deploy::pluralize(&resource.kind),
    };

    let api: Api<DynamicObject> = Api::namespaced_with(client.clone(), ns, &ar);
    match api.get(&resource.name).await {
        Ok(obj) => {
            let val = serde_json::to_value(&obj).unwrap_or_default();
            is_ready(&resource.kind, &val)
        }
        Err(_) => ResourceState::NotReady,
    }
}

pub async fn watch_resources(
    client: &kube::Client,
    resources: &[ResourceDescriptor],
    timeout_secs: u64,
) -> ResourceState {
    let timeout = Duration::from_secs(timeout_secs);
    let poll_interval = Duration::from_millis(500);

    let mut last_states: HashMap<String, ResourceState> = HashMap::new();

    let result = tokio::time::timeout(timeout, async {
        loop {
            let mut all_ready = true;
            let mut any_failed = false;

            for resource in resources {
                let key = format!("{}/{}", resource.kind, resource.name);
                let state = check_resource(client, resource).await;

                let prev = last_states.get(&key);
                if prev.is_none_or(|p| *p != state) {
                    eprintln!("[boom] {key} -> {state}");
                    last_states.insert(key.clone(), state.clone());
                }

                match &last_states[&key] {
                    ResourceState::Failed => any_failed = true,
                    ResourceState::NotReady => all_ready = false,
                    ResourceState::Ready => {}
                }
            }

            if any_failed {
                return ResourceState::Failed;
            }
            if all_ready {
                return ResourceState::Ready;
            }

            tokio::time::sleep(poll_interval).await;
        }
    })
    .await;

    match result {
        Ok(state) => state,
        Err(_) => ResourceState::NotReady, // timeout
    }
}

pub async fn collect_diagnostics(client: &kube::Client, resource: &ResourceDescriptor) -> String {
    let ns = resource.namespace.as_deref().unwrap_or("default");

    if resource.kind == "Pod" {
        let api: Api<Pod> = Api::namespaced(client.clone(), ns);
        match api.logs(&resource.name, &LogParams::default()).await {
            Ok(logs) => return format!("--- Logs for Pod/{} ---\n{logs}", resource.name),
            Err(e) => {
                return format!(
                    "--- Failed to fetch logs for Pod/{}: {e} ---",
                    resource.name
                );
            }
        }
    }

    // For other kinds, list events filtered by involved object name
    let api: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), ns);
    let field_selector = format!("involvedObject.name={}", resource.name);
    let lp = ListParams::default().fields(&field_selector);
    match api.list(&lp).await {
        Ok(event_list) => {
            let mut out = format!("--- Events for {}/{} ---\n", resource.kind, resource.name);
            for event in &event_list.items {
                let reason = event.reason.as_deref().unwrap_or("Unknown");
                let message = event.message.as_deref().unwrap_or("");
                let _ = writeln!(out, "  {reason}: {message}");
            }
            out
        }
        Err(e) => format!(
            "--- Failed to fetch events for {}/{}: {e} ---",
            resource.kind, resource.name
        ),
    }
}

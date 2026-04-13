use boom::monitor::{ResourceState, is_ready};
use serde_json::json;

#[test]
fn deployment_ready_when_available_equals_desired() {
    let status = json!({
        "status": { "availableReplicas": 3, "updatedReplicas": 3 },
        "spec": { "replicas": 3 }
    });
    assert_eq!(is_ready("Deployment", &status), ResourceState::Ready);
}

#[test]
fn deployment_not_ready_when_available_less_than_desired() {
    let status = json!({
        "status": { "availableReplicas": 1, "updatedReplicas": 3 },
        "spec": { "replicas": 3 }
    });
    assert_eq!(is_ready("Deployment", &status), ResourceState::NotReady);
}

#[test]
fn deployment_not_ready_when_updated_less_than_desired() {
    let status = json!({
        "status": { "availableReplicas": 3, "updatedReplicas": 1 },
        "spec": { "replicas": 3 }
    });
    assert_eq!(is_ready("Deployment", &status), ResourceState::NotReady);
}

#[test]
fn statefulset_ready_when_ready_replicas_equals_replicas() {
    let status = json!({
        "status": { "readyReplicas": 3 },
        "spec": { "replicas": 3 }
    });
    assert_eq!(is_ready("StatefulSet", &status), ResourceState::Ready);
}

#[test]
fn daemonset_ready_when_desired_equals_number_ready() {
    let status = json!({
        "status": { "desiredNumberScheduled": 5, "numberReady": 5 }
    });
    assert_eq!(is_ready("DaemonSet", &status), ResourceState::Ready);
}

#[test]
fn pod_ready_when_running_and_all_containers_ready() {
    let status = json!({
        "status": {
            "phase": "Running",
            "containerStatuses": [
                { "ready": true, "state": {} },
                { "ready": true, "state": {} }
            ]
        }
    });
    assert_eq!(is_ready("Pod", &status), ResourceState::Ready);
}

#[test]
fn pod_ready_when_succeeded() {
    let status = json!({
        "status": { "phase": "Succeeded" }
    });
    assert_eq!(is_ready("Pod", &status), ResourceState::Ready);
}

#[test]
fn pod_failed_on_crashloopbackoff() {
    let status = json!({
        "status": {
            "phase": "Running",
            "containerStatuses": [
                {
                    "ready": false,
                    "state": { "waiting": { "reason": "CrashLoopBackOff" } }
                }
            ]
        }
    });
    assert_eq!(is_ready("Pod", &status), ResourceState::Failed);
}

#[test]
fn pod_failed_on_imagepullbackoff() {
    let status = json!({
        "status": {
            "phase": "Pending",
            "containerStatuses": [
                {
                    "ready": false,
                    "state": { "waiting": { "reason": "ImagePullBackOff" } }
                }
            ]
        }
    });
    assert_eq!(is_ready("Pod", &status), ResourceState::Failed);
}

#[test]
fn job_ready_when_succeeded_equals_completions() {
    let status = json!({
        "status": { "succeeded": 3, "failed": 0 },
        "spec": { "completions": 3, "backoffLimit": 6 }
    });
    assert_eq!(is_ready("Job", &status), ResourceState::Ready);
}

#[test]
fn job_failed_when_failed_exceeds_backoff_limit() {
    let status = json!({
        "status": { "succeeded": 0, "failed": 7 },
        "spec": { "completions": 3, "backoffLimit": 6 }
    });
    assert_eq!(is_ready("Job", &status), ResourceState::Failed);
}

#[test]
fn always_ready_configmap() {
    assert_eq!(is_ready("ConfigMap", &json!({})), ResourceState::Ready);
}

#[test]
fn always_ready_secret() {
    assert_eq!(is_ready("Secret", &json!({})), ResourceState::Ready);
}

#[test]
fn always_ready_pvc() {
    assert_eq!(
        is_ready("PersistentVolumeClaim", &json!({})),
        ResourceState::Ready
    );
}

#[test]
fn always_ready_service() {
    assert_eq!(is_ready("Service", &json!({})), ResourceState::Ready);
}

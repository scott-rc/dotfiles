use boom::prune::{identify_stale, ResourceDescriptor};

fn descriptor(name: &str, kind: &str, namespace: &str) -> ResourceDescriptor {
    ResourceDescriptor {
        name: name.to_string(),
        kind: kind.to_string(),
        namespace: namespace.to_string(),
    }
}

#[test]
fn stale_resources_identified() {
    let deployed = vec![
        descriptor("a", "Deployment", "default"),
        descriptor("b", "Deployment", "default"),
    ];
    let existing = vec![
        descriptor("a", "Deployment", "default"),
        descriptor("b", "Deployment", "default"),
        descriptor("c", "Deployment", "default"),
    ];

    let stale = identify_stale(&deployed, &existing);
    assert_eq!(stale.len(), 1, "expected 1 stale resource, got {}", stale.len());
    assert_eq!(stale[0].name, "c");
}

#[test]
fn no_stale_resources() {
    let deployed = vec![
        descriptor("a", "Deployment", "default"),
        descriptor("b", "Deployment", "default"),
    ];
    let existing = vec![
        descriptor("a", "Deployment", "default"),
        descriptor("b", "Deployment", "default"),
    ];

    let stale = identify_stale(&deployed, &existing);
    assert!(stale.is_empty(), "expected no stale resources, got {}", stale.len());
}

#[test]
fn all_cluster_resources_stale() {
    let deployed: Vec<ResourceDescriptor> = vec![];
    let existing = vec![
        descriptor("a", "Deployment", "default"),
        descriptor("b", "Deployment", "default"),
    ];

    let stale = identify_stale(&deployed, &existing);
    assert_eq!(stale.len(), 2, "expected 2 stale resources, got {}", stale.len());
}

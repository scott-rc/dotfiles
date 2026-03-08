use boom::restart;

#[test]
fn build_restart_patch_sets_annotation_on_pod_template() {
    let patch = restart::build_restart_patch("2026-03-07T00:00:00Z");
    let annotation = patch
        .get("spec")
        .and_then(|s| s.get("template"))
        .and_then(|t| t.get("metadata"))
        .and_then(|m| m.get("annotations"))
        .and_then(|a| a.get("kubectl.kubernetes.io/restartedAt"))
        .and_then(|v| v.as_str());
    assert_eq!(
        annotation,
        Some("2026-03-07T00:00:00Z"),
        "restartedAt annotation should match the provided timestamp"
    );
}

#[test]
fn build_restart_patch_has_no_top_level_metadata() {
    let patch = restart::build_restart_patch("2026-03-07T00:00:00Z");
    assert!(
        patch.get("metadata").is_none(),
        "patch should not contain a top-level metadata key"
    );
}

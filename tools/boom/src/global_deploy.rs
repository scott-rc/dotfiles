use std::process;

use crate::deploy;
use crate::manifest::{self, ResourceDescriptor};

pub async fn run(
    client: kube::Client,
    mut resources: Vec<ResourceDescriptor>,
    verify_result: bool,
    global_timeout: u64,
) {
    let namespaced: Vec<_> = resources
        .iter()
        .filter(|r| !manifest::is_cluster_scoped(&r.kind))
        .map(|r| format!("{}/{}", r.kind, r.name))
        .collect();

    if !namespaced.is_empty() {
        eprintln!(
            "boom: global-deploy rejects namespaced resources: {}",
            namespaced.join(", ")
        );
        process::exit(1);
    }

    manifest::sort_by_priority(&mut resources);

    let ok = deploy::run(client, "", resources, verify_result, global_timeout).await;
    if ok {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

# boom

Kubernetes deploy tool with templated manifest rendering, tier-based resource ordering, health verification, and stale resource pruning.

Built by the parent `apply.sh` and symlinked to `~/.cargo/bin/boom`.

## Usage

### deploy

```
boom deploy --namespace <NS> --dir <DIR> [OPTIONS]
```

Renders templates from `--dir`, applies resources to the cluster in tier order, and optionally verifies readiness.

| Flag | Description |
|------|-------------|
| `-n`, `--namespace <NS>` | Target namespace (required) |
| `--dir <DIR>` | Directory containing YAML/template files (required) |
| `--values <FILE>` | YAML file with template bindings |
| `--context <CTX>` | Kubeconfig context |
| `--selector <SEL>` | Label selector |
| `--global-timeout <SECS>` | Readiness timeout (default: 300) |
| `--verify-result <BOOL>` | Wait for readiness (default: true) |
| `--prune` | Delete stale resources not in current manifests |

### global-deploy

```
boom global-deploy --dir <DIR> [OPTIONS]
```

Like `deploy` but only applies cluster-scoped resources (rejects namespaced). Same options as `deploy` except `--namespace`.

### restart

```
boom restart --namespace <NS> [OPTIONS]
```

Patches workloads with a `kubectl.kubernetes.io/restartedAt` annotation to trigger rolling restarts.

| Flag | Description |
|------|-------------|
| `-n`, `--namespace <NS>` | Target namespace (required) |
| `--context <CTX>` | Kubeconfig context |
| `--deployments <CSV>` | Comma-separated Deployment names |
| `--statefulsets <CSV>` | Comma-separated StatefulSet names |
| `--daemonsets <CSV>` | Comma-separated DaemonSet names |
| `--global-timeout <SECS>` | Readiness timeout (default: 300) |
| `--verify-result <BOOL>` | Wait for readiness (default: true) |

### render

```
boom render [OPTIONS]
```

Renders templates to stdout without applying. Useful for CI and debugging.

| Flag | Description |
|------|-------------|
| `--template-dir <DIR>` | Template directory (default: `.`) |
| `--bindings <KEY=VALUE>` | Template binding (repeatable) |
| `--bindings-file <FILE>` | YAML file with template bindings |
| `--current-sha` | Inject `current_sha` from `git rev-parse HEAD` |

## Features

- **Template rendering** -- Jinja2-style templating (minijinja) for `.yml.j2`/`.yaml.j2` files; plain `.yml`/`.yaml` files pass through unchanged
- **Tier-based ordering** -- deploys resources in priority order: Tier 0 (Namespace, ServiceAccount, CRD), Tier 1 (ConfigMap, Secret, PVC, StorageClass), Tier 2 (everything else)
- **Dynamic resources** -- applies arbitrary Kubernetes resources including custom resources via dynamic client (`Api<DynamicObject>`)
- **Health verification** -- polls resources until ready or timeout with kind-specific readiness checks (Deployment, StatefulSet, DaemonSet, Pod, Job)
- **Terminal failure detection** -- immediately exits on CrashLoopBackOff, ImagePullBackOff, and similar terminal states
- **Diagnostics** -- collects pod logs and events on failure for debugging
- **Pruning** -- identifies and deletes resources from previous deployments not present in current manifests
- **Colored output** -- info/success/warn/error messages and formatted summary tables

## Architecture

**Deploy pipeline**: Loads templates from a directory, renders with minijinja (strict mode -- undefined variables error), parses multi-document YAML into `ResourceDescriptor` structs, classifies into tiers, then applies each tier in order using `tokio::task::JoinSet` for parallel resource application within a tier. Optionally polls for readiness and prunes stale resources.

**Readiness checking**: Kind-specific logic -- Deployments check replica counts, StatefulSets check ready replicas, DaemonSets compare desired vs scheduled nodes, Pods check phase and container statuses, Jobs check completion counts. Polls every 500ms up to the global timeout.

## Modules

| Module | Purpose |
|--------|---------|
| `main.rs` | CLI parsing (clap), command dispatch, template loading orchestration |
| `client.rs` | Kubeconfig reading, Kubernetes client construction |
| `render.rs` | Jinja2 template loading/rendering, bindings file/CLI parsing |
| `manifest.rs` | Multi-document YAML parsing, `ResourceDescriptor`, tier classification |
| `deploy.rs` | Resource application via dynamic client, API version parsing, kind pluralization |
| `monitor.rs` | Readiness polling, kind-specific health checks, diagnostics collection |
| `global_deploy.rs` | Cluster-scoped deployment (rejects namespaced resources) |
| `restart.rs` | Workload restart via annotation patching |
| `prune.rs` | Stale resource identification and deletion |
| `output.rs` | Colored terminal output and formatted summary tables |

## Build

```bash
cargo build --release   # from tools/boom/
```

## Testing

```bash
cargo test              # from tools/boom/
```

Tests live in `tests/` covering template rendering, manifest parsing, readiness checks, pruning, restart, and global deploy. Uses `tempfile` for fixture directories.
